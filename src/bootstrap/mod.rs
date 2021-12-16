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

use serde::Deserialize;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc::Receiver,
};
use tokio_socks::tcp::Socks5Stream;

use crate::bootstrap::{
    mojang::MojangApi,
    storage::{ProducedUser, ValidUser},
};

pub mod block_data;
pub mod csv;
pub mod dns;
pub mod mojang;
pub mod opts;
pub mod storage;

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
    pub mojang: MojangApi,
    pub read: OwnedReadHalf,
    pub write: OwnedWriteHalf,
}

impl Connection {
    pub fn stream(
        address: Address,
        mut users: tokio::sync::mpsc::Receiver<ProducedUser>,
    ) -> Receiver<Connection> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::task::spawn_local(async move {
            while let Some(user) = users.recv().await {
                let tx = tx.clone();
                let address = address.clone();
                tokio::task::spawn_local(async move {
                    let ProducedUser {
                        proxy,
                        user,
                        mojang,
                    } = user;

                    let target = String::from(&address);

                    let conn = match proxy {
                        Some(proxy) => {
                            let conn = Socks5Stream::connect_with_password(
                                proxy.address().as_str(),
                                target.as_str(),
                                &proxy.user,
                                &proxy.pass,
                            )
                            .await
                            .unwrap();
                            conn.into_inner()
                        }
                        None => TcpStream::connect(target.as_str()).await.unwrap(),
                    };

                    let (read, write) = conn.into_split();
                    tx.send(Connection {
                        user,
                        address,
                        mojang,
                        read,
                        write,
                    })
                    .await
                    .unwrap();
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
