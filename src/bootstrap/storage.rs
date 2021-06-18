use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use packets::types::UUID;
use serde::{Deserialize, Serialize};

use crate::bootstrap::{CSVUser, Proxy};
use crate::bootstrap::mojang::{AuthResponse, Mojang};
use crate::error::Error;

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
struct ValidUser {
    email: String,
    username: String,
    password: String,
    last_checked: u128,
    uuid: UUID,
    access_id: String,
    client_id: String,
}

impl User {
    fn email(&self) -> &String {
        match self {
            User::Valid(ValidUser { email, .. }) => email,
            User::Invalid(InvalidUser { email, .. }) => email
        }
    }
}

struct UserCache {
    file_path: PathBuf,
    cache: HashMap<String, User>,
}


async fn validate_user(user: &ValidUser, proxy: &Proxy) {}

fn time() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
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

    fn get_or_put(&mut self, user: &CSVUser, iter: &mut impl Iterator<Item=&Proxy>) -> &User {
        match self.cache.get_mut(&user.email) {
            None => {
                let proxy = iter.next().unwrap();
                let mojang = Mojang::socks5(proxy).unwrap();
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
                        self.cache.entry(valid.email.clone()).insert(User::Valid(valid_user)).get()
                    }

                    // we cannot do anything more -> change to invalid
                    Err(_) => {
                        let invalid = InvalidUser {
                            email: user.email.clone(),
                            password: user.password.clone(),
                        };
                        self.cache.entry(invalid.email.clone()).insert(User::Invalid(invalid)).get()
                    }
                }
            }
            Some(cached) => {
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
                cached
            }
        }
    }

    pub fn obtain_users(&mut self, count: usize, users: &Vec<CSVUser>, proxies: &Vec<Proxy>) -> Vec<ValidUser> {
        let mut valid_users = Vec::with_capacity(count);

        let mut proxies = proxies.iter().cycle();

        for csv_user in users {
            match self.get_or_put(user, &mut proxies) {
                User::Valid(valid_user) => {
                    valid_users.push(valid_user.clone());
                    if valid_users.len() >= count {
                        return valid_users;
                    }
                }
                User::Invalid(_) => {}
            }
        }

        valid_users
    }
}
