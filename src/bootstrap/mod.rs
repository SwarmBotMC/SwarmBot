/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use serde::Deserialize;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::Receiver;
use tokio_socks::tcp::Socks5Stream;

use crate::bootstrap::mojang::Mojang;
use crate::bootstrap::storage::{ProxyUser, ValidUser};

pub mod opts;
pub mod csv;
pub mod block_data;
pub mod dns;
pub mod storage;
pub mod mojang;


#[derive(Clone, Debug)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

impl From<&Address> for String {
    fn from(addr: &Address) -> Self {
        format!("{}:{}", addr.host, addr.port)
    }
}

#[derive(Debug)]
pub struct Connection {
    pub user: ValidUser,
    pub address: Address,
    pub mojang: Mojang,
    pub read: OwnedReadHalf,
    pub write: OwnedWriteHalf,
}

impl Connection {
    pub fn stream(address: Address, mut users: tokio::sync::mpsc::Receiver<ProxyUser>) -> Receiver<Connection> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::task::spawn_local(async move {
            while let Some(user) = users.recv().await {
                let tx = tx.clone();
                let address = address.clone();
                tokio::task::spawn_local(async move {
                    let ProxyUser { proxy, user, mojang } = user;
                    let target = String::from(&address);
                    let conn = Socks5Stream::connect_with_password(proxy.address().as_str(), target.as_str(), &proxy.user, &proxy.pass).await.unwrap();
                    let (read, write) = conn.into_inner().into_split();
                    tx.send(Connection {
                        user,
                        address,
                        mojang,
                        read,
                        write,
                    }).await.unwrap();
                });
            }
        });

        rx
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CSVUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proxy {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub pass: String,
}

impl Proxy {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
