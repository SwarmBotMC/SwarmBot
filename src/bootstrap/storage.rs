// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use bincode::{config::Configuration, Decode, Encode};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::mpsc::Receiver;

use crate::{
    bootstrap,
    bootstrap::{mojang::MojangApi, CSVUser, Proxy},
    HasContext, ResContext,
};

#[derive(Encode, Decode, Debug)]
struct Root {
    users: Vec<User>,
}

#[derive(Encode, Decode, Debug)]
enum User {
    Valid(ValidUser),
    Invalid(InvalidUser),
}

#[derive(Encode, Decode, Debug)]
struct InvalidUser {
    email: String,
    password: String,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct ValidUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub last_checked: u64,
    pub uuid: String,
    pub access_id: String,
    pub client_id: String,
}

impl User {
    fn email(&self) -> &String {
        match self {
            User::Valid(ValidUser { email, .. }) => email,
            User::Invalid(InvalidUser { email, .. }) => email,
        }
    }
}

pub struct UserCache {
    file_path: PathBuf,
    cache: HashMap<String, User>,
}

/// A bot data holds the "Mojang" object used in cache to verify that the user
/// is valid along with data about what the proxy address is and the valid user
/// information
#[derive(Debug)]
pub struct BotData {
    pub user: ValidUser,
    pub proxy: Option<Proxy>,
    pub mojang: MojangApi,
}

impl BotData {
    pub fn load(
        proxy: bool,
        users_file: &str,
        proxies_file: &str,
        count: usize,
    ) -> ResContext<Receiver<BotData>> {
        let csv_file = File::open(&users_file)
            .context(|| format!("could not open users file {}", users_file))?;

        let csv_users =
            bootstrap::csv::read_users(csv_file).context_str("could not open users file")?;

        let proxies = match proxy {
            true => {
                let proxies_file = File::open(&proxies_file)
                    .context(|| format!("could not open proxies file {}", proxies_file))?;
                bootstrap::csv::read_proxies(proxies_file)
                    .context_str("could not open proxies file")?
                    .into_iter()
                    .map(Some)
                    .collect()
            }
            false => {
                vec![None]
            }
        };

        let cache = UserCache::load("cache.db".into());

        Ok(cache.obtain_users(count, csv_users, proxies))
    }
}

fn time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

impl UserCache {
    pub fn load(file_path: PathBuf) -> UserCache {
        let exists = std::fs::try_exists(&file_path).unwrap();
        if !exists {
            UserCache {
                file_path,
                cache: HashMap::new(),
            }
        } else {
            let file = File::open(&file_path).unwrap();
            let bytes: Result<Vec<_>, _> = file.bytes().collect();
            let bytes = bytes.unwrap();
            let (Root { users }, _) =
                bincode::decode_from_slice(&bytes, Configuration::standard()).unwrap();

            let cache: HashMap<_, _> = users
                .into_iter()
                .map(|user| (user.email().clone(), user))
                .collect();
            UserCache { file_path, cache }
        }
    }

    async fn get_or_put(
        &mut self,
        user: &CSVUser,
        iter: &mut impl Iterator<Item = Option<Proxy>>,
    ) -> Option<(MojangApi, Option<Proxy>, ValidUser)> {
        match self.cache.get_mut(&user.email) {
            None => {
                let proxy = iter.next().unwrap();
                let mojang = MojangApi::try_from(proxy.as_ref()).unwrap();
                match mojang.authenticate(&user.email, &user.password).await {
                    Ok(res) => {
                        let valid_user = ValidUser {
                            email: user.email.clone(),
                            username: res.username,
                            password: user.password.clone(),
                            last_checked: time(),
                            uuid: res.uuid.to_string(),
                            access_id: res.access_token,
                            client_id: res.client_token,
                        };
                        let valid_user = valid_user;
                        self.cache
                            .insert(valid_user.email.clone(), User::Valid(valid_user.clone()));
                        Some((mojang, proxy, valid_user))
                    }

                    // we cannot do anything more -> change to invalid
                    Err(e) => {
                        println!("failed authentication for {} .. {}", user.email, e);
                        let invalid = InvalidUser {
                            email: user.email.clone(),
                            password: user.password.clone(),
                        };
                        self.cache
                            .insert(invalid.email.clone(), User::Invalid(invalid));
                        None
                    }
                }
            }
            Some(cached) => {
                match cached {
                    User::Valid(valid) => {
                        let proxy = iter.next().unwrap();
                        let mojang = MojangApi::try_from(proxy.as_ref()).unwrap();

                        // if verified in last day don't even check to verify
                        if time() - valid.last_checked < 3600 * 24 {
                            return Some((mojang, proxy, valid.clone()));
                        }

                        println!("refreshing auth tokens for {} due to time", user.email);

                        let is_valid = mojang
                            .validate(&valid.access_id, &valid.client_id)
                            .await
                            .unwrap();

                        if !is_valid {
                            println!("failed validating {}", user.email);
                            match mojang.refresh(&valid.access_id, &valid.client_id).await {
                                Ok(auth) => {
                                    valid.access_id = auth.access_token;
                                    valid.username = auth.username;
                                    valid.uuid = auth.uuid.to_string();
                                    valid.client_id = auth.client_token;
                                    valid.last_checked = time();
                                    return Some((mojang, proxy, valid.clone()));
                                }

                                // we could not refresh -> try to authenticate
                                Err(e) => {
                                    println!("failed refreshing {} .. {}", user.email, e);
                                    match mojang.authenticate(&valid.email, &valid.password).await {
                                        Ok(auth) => {
                                            valid.access_id = auth.access_token;
                                            valid.username = auth.username;
                                            valid.uuid = auth.uuid.to_string();
                                            valid.client_id = auth.client_token;
                                            valid.last_checked = time();
                                            return Some((mojang, proxy, valid.clone()));
                                        }

                                        // we cannot do anything more -> change to invalid
                                        Err(e) => {
                                            println!(
                                                "failed authenticating {} .. {}",
                                                user.email, e
                                            );
                                            *cached = User::Invalid(InvalidUser {
                                                email: valid.email.clone(),
                                                password: valid.password.clone(),
                                            })
                                        }
                                    }
                                }
                            }
                        } else {
                            return Some((mojang, proxy, valid.clone()));
                        }
                    }
                    User::Invalid(_invalid) => {}
                }

                println!("user {} is cached as invalid. If this user **is** valid, delete cache.db and re-run", user.email);
                None
            }
        }
    }

    pub fn obtain_users(
        mut self,
        count: usize,
        users: Vec<CSVUser>,
        proxies: Vec<Option<Proxy>>,
    ) -> Receiver<BotData> {
        let mut proxies = proxies.into_iter().cycle();

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::task::spawn_local(async move {
            let mut local_count = 0;

            'user_loop: for csv_user in users.into_iter() {
                if let Some((mojang, proxy, user)) = self.get_or_put(&csv_user, &mut proxies).await
                {
                    local_count += 1;
                    println!("valid user {}", user.email);
                    tx.send(BotData {
                        user,
                        proxy,
                        mojang,
                    })
                    .await
                    .unwrap();
                } else {
                    println!("invalid user {}", csv_user.email);
                }

                if local_count >= count {
                    break 'user_loop;
                }
            }

            let mut file = OpenOptions::new()
                .create(true)
                .append(false)
                .write(true)
                .open(&self.file_path)
                .unwrap();

            let users = self.cache.drain().map(|(_, v)| v).collect();

            let root = Root { users };

            let data = bincode::encode_to_vec(&root, Configuration::standard()).unwrap();
            file.write_all(&data).unwrap();
            file.flush().unwrap();
        });

        rx
    }
}
