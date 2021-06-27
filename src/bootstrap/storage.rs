/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/27/21, 3:15 PM
 */


use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};



use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;


use crate::bootstrap::{CSVUser, Proxy};
use crate::bootstrap::mojang::{Mojang};


#[derive(Serialize, Deserialize, Debug)]
struct Root {
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
enum User {
    Valid(ValidUser),
    Invalid(InvalidUser),
}

#[derive(Serialize, Deserialize, Debug)]
struct InvalidUser {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
            User::Invalid(InvalidUser { email, .. }) => email
        }
    }
}

pub struct UserCache {
    file_path: PathBuf,
    cache: HashMap<String, User>,
}


/// A proxy user holds the "Mojang" object used in cache to verify that the user is valid along with
/// data about what the proxy address is and the valid user information
#[derive(Debug)]
pub struct ProxyUser {
    pub user: ValidUser,
    pub proxy: Proxy,
    pub mojang: Mojang,
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
            let Root{users} = bincode::deserialize(&bytes).unwrap();

            let cache: HashMap<_, _> = users.into_iter().map(|user| (user.email().clone(), user)).collect();
            UserCache {
                file_path,
                cache,
            }
        }
    }

    async fn get_or_put(&mut self, user: &CSVUser, iter: &mut impl Iterator<Item=Proxy>) -> Option<(Mojang, Proxy, ValidUser)> {
        match self.cache.get_mut(&user.email) {
            None => {
                let proxy = iter.next().unwrap();
                let mojang = Mojang::socks5(&proxy).unwrap();
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
                        self.cache.insert(valid_user.email.clone(), User::Valid(valid_user.clone()));
                        Some((mojang, proxy, valid_user))
                    }

                    // we cannot do anything more -> change to invalid
                    Err(e) => {
                        println!("failed authentication for {} .. {}", user.email, e);
                        let invalid = InvalidUser {
                            email: user.email.clone(),
                            password: user.password.clone(),
                        };
                        self.cache.insert(invalid.email.clone(), User::Invalid(invalid));
                        None
                    }
                }
            }
            Some(cached) => {
                match cached {
                    User::Valid(valid) => {
                        let proxy = iter.next().unwrap();
                        let mojang = Mojang::socks5(&proxy).unwrap();

                        // if verified in last day don't even check to verify
                        if time() - valid.last_checked < 3600 * 24 {
                            return Some((mojang, proxy, valid.clone()));
                        }

                        let is_valid = mojang.validate(&valid.access_id, &valid.client_id).await.unwrap();

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
                                            println!("failed authenticating {} .. {}", user.email, e);
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
                None
            }
        }
    }

    pub fn obtain_users(mut self, count: usize, users: Vec<CSVUser>, proxies: Vec<Proxy>) -> Receiver<ProxyUser> {
        let mut proxies = proxies.into_iter().cycle();

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::task::spawn_local(async move {
            let mut local_count = 0;

            'user_loop:
            for csv_user in users.into_iter() {
                if let Some((mojang, proxy, user)) = self.get_or_put(&csv_user, &mut proxies).await {
                    local_count += 1;
                    println!("valid user {}", user.email);
                    tx.send(ProxyUser {
                        user,
                        proxy,
                        mojang,
                    }).await.unwrap();
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

            let root = Root {
                users
            };

            let data = bincode::serialize(&root).unwrap();
            file.write_all(&data).unwrap();
            file.flush().unwrap();

        });

        rx
    }
}
