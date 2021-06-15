use std::fs::File;

use rand::seq::SliceRandom;
use serde::Deserialize;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::bootstrap::csv::read_users;
use crate::bootstrap::dns::dns_lookup;
use crate::bootstrap::mojang::{AuthResponse, Mojang};
use crate::bootstrap::opts::Opts;
use crate::bootstrap::tcp::obtain_connections;
use crate::db::{Db, ValidDbUser, CachedUser};
use crate::error::{err, Error, HasContext, ResContext};
use std::time::Duration;
use itertools::Itertools;
use packets::types::UUID;

mod opts;
mod csv;
mod tcp;
mod dns;
pub mod mojang;


pub struct Address {
    host: String,
    port: u16,
}

#[derive(Debug)]
pub struct Connection {
    pub user: CachedUser,
    pub host: String,
    pub port: u16,
    pub mojang: Mojang,
    pub read: OwnedReadHalf,
    pub write: OwnedWriteHalf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
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

pub struct Output {
    pub version: usize,
    pub delay_millis: u64,
    pub connections: tokio::sync::mpsc::Receiver<Connection>,
}

pub async fn init() -> ResContext<Output> {

    // read from config
    let Opts { users_file, proxy, proxies_file, host, count, version, port, db, delay , ..} = Opts::get();



    // DNS Lookup
    let Address { host, port } = dns_lookup(&host).await.unwrap_or(Address {
        host,
        port,
    });

    // list users we want to login
    let mut users = {
        let file = File::open(&users_file).context(|| format!("opening users ({})", users_file))?;
        read_users(file).context(|| format!("reading users ({})", users_file))?
    };

    let users = users.into_iter().skip(87).collect_vec();

    let db = Db::init().await;

    // the connections
    let connections = obtain_connections(proxy, &proxies_file, &host, port, count, &db).await?;

    let output = Output {
        version,
        delay_millis: delay,
        connections,
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::db::Db;
    use crate::bootstrap::mojang::Mojang;

    #[tokio::test]
    async fn update_db(){
        println!("start");
        let users = super::csv::read_users(File::open("users.csv").unwrap()).unwrap();
        let db = Db::init().await;
        let mojang = Mojang::default();
        db.update_users(&users, &mojang).await;
    }
}
