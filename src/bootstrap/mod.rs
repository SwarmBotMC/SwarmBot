use std::fs::File;

use serde::Deserialize;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::bootstrap::csv::read_users;
use crate::bootstrap::dns::dns_lookup;
use crate::bootstrap::mojang::Mojang;
use crate::bootstrap::opts::Opts;
use crate::bootstrap::tcp::obtain_connections;
use crate::error::{err, HasContext, ResContext};
use rand::seq::SliceRandom;

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
    pub user: User,
    pub online: bool,
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
    pub online: bool,
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
    pub connections: tokio::sync::mpsc::Receiver<Connection>,
}

pub async fn init() -> ResContext<Output> {

    // read from config
    let Opts { users_file, proxy, proxies_file, host, count, version, port, .. } = Opts::get();


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

    // we are requesting too many users
    if users.len() < count {
        err(format!("there are {} users but {} were requested", users.len(), count))?
    }

    println!("starting with {} users", count);

    // the users we will use


    users.shuffle(&mut rand::thread_rng());
    let users = &users[..count];

    // the connections
    let connections = obtain_connections(proxy, &proxies_file, &host, port, users).await?;

    let output = Output {
        version,
        connections,
    };

    Ok(output)
}
