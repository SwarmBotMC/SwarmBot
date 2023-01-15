//! used for boostrapping the code
use anyhow::Context;
use futures::{Stream, StreamExt};
use serde::Deserialize;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc::Receiver,
};
use tokio_socks::tcp::Socks5Stream;

use crate::bootstrap::storage::{BotConnectionData, BotData};

pub mod csv;
pub mod dns;
pub mod mojang;
pub mod opts;
pub mod storage;

/// A server address
#[derive(Clone, Debug)]
pub struct Address {
    /// the hostname
    pub host: String,
    /// the port
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
    pub bot: BotData,

    /// the address being logged into
    pub server_address: Address,

    /// A read stream (from the server)
    pub read: OwnedReadHalf,

    /// A write stream (to the server)
    pub write: OwnedWriteHalf,
}

/// Obtain a concrete TCP connection to the sever `address`. This only
/// establishes a connection and does not anything involving
async fn obtain_connection(
    user: BotConnectionData,
    server_address: Address,
) -> anyhow::Result<BotConnection> {
    let BotConnectionData { bot, proxy } = user;

    let target = String::from(&server_address);

    let conn = if let Some(proxy) = proxy {
        let address = proxy.address();
        let conn = Socks5Stream::connect_with_password(
            proxy.address().as_str(),
            target.as_str(),
            &proxy.user,
            &proxy.pass,
        )
        .await
        .with_context(|| format!("could not create to socks {address}"))?;

        conn.into_inner()
    } else {
        TcpStream::connect(target.as_str()).await.unwrap()
    };

    let (read, write) = conn.into_split();
    Ok(BotConnection {
        bot,
        server_address,
        read,
        write,
    })
}

impl BotConnection {
    /// Generates connections given [`BotConnectionData`] and an address
    pub fn stream(
        server_address: Address,
        mut users: impl Stream<Item = BotConnectionData> + Unpin + 'static,
    ) -> Receiver<anyhow::Result<Self>> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::task::spawn_local(async move {
            while let Some(user) = users.next().await {
                let tx = tx.clone();
                let address = server_address.clone();
                tokio::task::spawn_local(async move {
                    let connection = obtain_connection(user, address).await;
                    tx.send(connection).await.unwrap();
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
