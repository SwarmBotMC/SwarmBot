use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use packets::types::UUID;
use serde::{Deserialize, Serialize};

use crate::bootstrap::{CSVUser, Proxy};
use crate::bootstrap::mojang::{AuthResponse, Mojang};
use crate::error::Error;
use std::cmp::min;
use tokio::sync::mpsc::Receiver;
use tokio_socks::tcp::Socks5Stream;

#[derive(Serialize, Deserialize)]
struct Root {
    users: Vec<User>,
}

#[derive(Serialize, Deserialize)]
enum User {
    Valid(ValidUser),
    Invalid(InvalidUser),
}

#[derive(Serialize, Deserialize)]
struct InvalidUser {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ValidUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub last_checked: u128,
    pub uuid: UUID,
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
pub struct ProxyUser {
    pub user: ValidUser,
    pub proxy: Proxy,
    pub mojang: Mojang
}


async fn validate_user(user: &ValidUser, proxy: &Proxy) {}

fn time() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

impl Drop for UserCache {
    fn drop(&mut self) {
        let file = File::open(&self.file_path).unwrap();
        let mut s = flexbuffers::FlexbufferSerializer::new();
        // TODO:
    }
}

impl UserCache {

    pub fn load(file_path: PathBuf) -> UserCache {

        let file = File::open(&file_path).unwrap();
        let mut s = flexbuffers::FlexbufferSerializer::new();

        let r = flexbuffers::Reader::get_root(file).unwrap();
        let Root { users } = Root::deserialize(r).unwrap();

        let cache: HashMap<_, _> = users.into_iter().map(|user| (user.email().clone(), user)).collect();
        UserCache {
            file_path,
            cache,
        }
    }

    async fn get_or_put(&mut self, user: &CSVUser, iter: &mut impl Iterator<Item=Proxy>) -> (Option<Mojang>, Proxy, &User) {
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
                            uuid: res.uuiid,
                            access_id: res.access_id,
                            client_id: res.client_id,
                        };
                        (Some(mojang), proxy,self.cache.entry(valid.email.clone()).insert(User::Valid(valid_user)).get())
                    }

                    // we cannot do anything more -> change to invalid
                    Err(_) => {
                        let invalid = InvalidUser {
                            email: user.email.clone(),
                            password: user.password.clone(),
                        };
                        (Some(mojang), proxy, self.cache.entry(invalid.email.clone()).insert(User::Invalid(invalid)).get())
                    }
                }
            }
            Some(cached) => {
                let mut mojang_res = None;
                match cached {
                    User::Valid(valid) => {
                        let mojang = Mojang::socks5(proxy).unwrap();
                        let is_valid = mojang.validate(&valid.access_id, &valid.client_id).await.unwrap();
                        if !is_valid {
                            match mojang.refresh(&valid.access_id, &valid.client_id).await {
                                Ok(auth) => {
                                    valid.access_id = auth.access_token;
                                    valid.username = auth.username;
                                    valid.uuid = auth.uuid;
                                    valid.client_id = auth.client_token;
                                    valid.last_checked = time();
                                    mojang_res = Some(mojang);
                                }

                                // we could not refresh -> try to authenticate
                                Err(_) => {
                                    match mojang.authenticate(&valid.email, &valid.password).await {
                                        Ok(res) => {
                                            valid.access_id = auth.access_token;
                                            valid.username = auth.username;
                                            valid.uuid = auth.uuid;
                                            valid.client_id = auth.client_token;
                                            valid.last_checked = time();
                                            mojang_res = Some(mojang);
                                        }

                                        // we cannot do anything more -> change to invalid
                                        Err(_) => {
                                            *cached = User::Invalid(InvalidUser {
                                                email: valid.email.clone(),
                                                password: valid.password.clone(),
                                            })
                                        }
                                    }
                                }
                            }
                        }
                    }
                    User::Invalid(invalid) => {}
                }
                (mojang_res, proxy, cached)
            }
        }
    }

    pub fn obtain_users(&mut self, count: usize, users: Vec<CSVUser>, proxies: Vec<Proxy>) -> Receiver<ProxyUser> {
        let mut proxies = proxies.into_iter().cycle();

        let (tx, rx) = tokio::sync::mpsc::channel(1);

        tokio::task::spawn_local(async move {
            for csv_user in users {
                match self.get_or_put(user, &mut proxies).await {
                    (Some(mojang), proxy, User::Valid(valid_user)) => {
                        tx.send(ProxyUser {
                            user: valid_user.clone(),
                            proxy,
                            mojang
                        });
                        if valid_users.len() >= count {
                            return;
                        }
                    }
                    _ => {}
                }
            }

        });

        rx
    }
}
