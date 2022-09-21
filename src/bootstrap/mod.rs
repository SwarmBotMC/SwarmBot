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
    mojang::MojangClient,
    storage::{BotDataLoader, OnlineUser},
};

pub mod csv;
pub mod dns;
pub mod mojang;
pub mod opts;
pub mod storage;

/// A server address
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

/// Represents a connection of a bot to a server
#[derive(Debug)]
pub struct BotConnection {
    /// the user information
    pub user: OnlineUser,

    /// the address being logged into
    pub server_address: Address,

    /// the mojang client we can interact with mojang for
    pub mojang: MojangClient,

    /// A read stream (from the server)
    pub read: OwnedReadHalf,

    /// A write stream (to the server)
    pub write: OwnedWriteHalf,
}

impl BotConnection {
    /// Generates connections given BotData and an address
    pub fn stream(
        server_address: Address,
        mut users: tokio::sync::mpsc::Receiver<BotDataLoader>,
    ) -> Receiver<BotConnection> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::task::spawn_local(async move {
            while let Some(user) = users.recv().await {
                let tx = tx.clone();
                let address = server_address.clone();
                tokio::task::spawn_local(async move {
                    let BotDataLoader {
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
                    tx.send(BotConnection {
                        user,
                        server_address: address,
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
