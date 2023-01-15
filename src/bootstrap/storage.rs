//! Utils to store information

use std::{
    collections::HashMap,
    convert::TryFrom,
    fs,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use bincode::{Decode, Encode};
use swarm_bot_packets::types::UUID;
use tokio::sync::mpsc::Receiver;
use tokio_stream::Stream;

use crate::{
    bootstrap,
    bootstrap::{mojang::MojangClient, CSVUser, Proxy},
};

#[derive(Encode, Decode, Debug)]
struct Root {
    users: Vec<User>,
}

#[derive(Encode, Decode, Debug)]
enum User {
    Valid(OnlineUser),
    Invalid(InvalidUser),
}

#[derive(Encode, Decode, Debug)]
struct InvalidUser {
    email: String,
    password: String,
}

#[derive(Debug)]
pub struct OfflineUser {
    pub username: String,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct OnlineUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub last_checked: u64,
    pub uuid: String,
    pub access_id: String,
    pub client_id: String,
}

impl OnlineUser {
    pub fn uuid(&self) -> UUID {
        UUID::from(&self.uuid)
    }
}

impl User {
    const fn email(&self) -> &String {
        match self {
            Self::Invalid(InvalidUser { email, .. }) | Self::Valid(OnlineUser { email, .. }) => {
                email
            }
        }
    }
}

pub struct UserCache {
    file_path: PathBuf,
    cache: HashMap<String, User>,
}

#[derive(Debug)]
pub enum BotData {
    Online {
        /// the online user
        user: OnlineUser,

        /// the mojang client we can interact with mojang for
        mojang: MojangClient,
    },
    Offline {
        /// the invalid user
        user: OfflineUser,
    },
}

impl BotData {
    pub fn username(&self) -> &str {
        match self {
            Self::Online { user, .. } => &user.username,
            Self::Offline { user } => &user.username,
        }
    }
}

/// A bot data holds the "Mojang" object used in cache to verify that the user
/// is valid along with data about what the proxy address is and the valid user
/// information
#[derive(Debug)]
pub struct BotConnectionData {
    pub bot: BotData,
    pub proxy: Option<Proxy>,
}

impl BotConnectionData {
    pub fn offline_random() -> impl Stream<Item = Self> {
        let mut idx = 0;
        futures::stream::repeat_with(move || {
            let username = format!("Bot{idx:0>4}");
            let res = Self {
                bot: BotData::Offline {
                    user: OfflineUser { username },
                },
                proxy: None,
            };

            idx += 1;

            println!("generated offline {res:?}");

            res
        })
    }
    pub fn load_from_files(
        users_file: &str,
        proxies_file: &str,
        proxy: bool,
        count: usize,
    ) -> anyhow::Result<Receiver<Self>> {
        let csv_file = File::open(users_file)
            .with_context(|| format!("could not open users file {users_file}"))?;

        let csv_users =
            bootstrap::csv::read_users(csv_file).context("could not open users file")?;

        let proxies = if proxy {
            let proxies_file = File::open(proxies_file)
                .with_context(|| format!("could not open proxies file {proxies_file}"))?;
            bootstrap::csv::read_proxies(proxies_file)
                .context("could not open proxies file")?
                .into_iter()
                .map(Some)
                .collect()
        } else {
            vec![None]
        };

        let cache = UserCache::load("cache.db".into())?;

        Ok(cache.obtain_users(count, csv_users, proxies))
    }
}

fn time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    since_the_epoch.as_secs()
}

impl UserCache {
    pub fn load(file_path: PathBuf) -> anyhow::Result<Self> {
        let exists = fs::try_exists(&file_path)
            .with_context(|| format!("cannot load user from: {file_path:?} (DNE)"))?;
        let res = if exists {
            let file = File::open(&file_path).context("could not load file")?;
            let bytes: Result<Vec<_>, _> = file.bytes().collect();
            let bytes = bytes.context("could not load bytes")?;
            let config = bincode::config::standard();
            let (Root { users }, _) = bincode::decode_from_slice(&bytes, config)
                .context("could not decode from slice")?;

            let cache: HashMap<_, _> = users
                .into_iter()
                .map(|user| (user.email().clone(), user))
                .collect();
            Self { file_path, cache }
        } else {
            Self {
                file_path,
                cache: HashMap::new(),
            }
        };
        Ok(res)
    }

    /// Takes a [`CSVUser`] and returns the user's data along with the proxy
    /// associated with it
    async fn get_or_put(
        &mut self,
        user: &CSVUser,
        iter: &mut impl Iterator<Item = Option<Proxy>>,
    ) -> Option<(MojangClient, Option<Proxy>, OnlineUser)> {
        match self.cache.get_mut(&user.email) {
            None => {
                let proxy = iter.next().unwrap();
                let mojang = MojangClient::try_from(proxy.as_ref()).unwrap();
                match mojang.authenticate(&user.email, &user.password).await {
                    Ok(res) => {
                        let valid_user = OnlineUser {
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
                        let mojang = MojangClient::try_from(proxy.as_ref()).unwrap();

                        // if verified in last day don't even check to verify
                        if time() - valid.last_checked < 3600 * 24 {
                            return Some((mojang, proxy, valid.clone()));
                        }

                        println!("refreshing auth tokens for {} due to time", user.email);

                        let is_valid = mojang
                            .validate(&valid.access_id, &valid.client_id)
                            .await
                            .unwrap();

                        if is_valid {
                            return Some((mojang, proxy, valid.clone()));
                        }
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
                                        });
                                    }
                                }
                            }
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
    ) -> Receiver<BotConnectionData> {
        let mut proxies = proxies.into_iter().cycle();

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // spawn the receiver that will yield players
        tokio::task::spawn_local(async move {
            let mut local_count = 0;

            'user_loop: for csv_user in users {
                if let Some((mojang, proxy, user)) = self.get_or_put(&csv_user, &mut proxies).await
                {
                    local_count += 1;
                    println!("valid user {}", user.email);
                    tx.send(BotConnectionData {
                        bot: BotData::Online { user, mojang },
                        proxy,
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

            let data = bincode::encode_to_vec(&root, bincode::config::standard()).unwrap();
            file.write_all(&data).unwrap();
            file.flush().unwrap();
        });

        rx
    }
}
