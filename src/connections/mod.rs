use std::fs::File;

use crate::connections::csv::read_users;
use crate::connections::opts::Opts;
use crate::connections::tcp::{obtain_connections};
use serde::Deserialize;
use crate::error::{err, HasContext, ResContext};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

mod opts;
mod csv;
mod tcp;

#[derive(Debug)]
pub struct Connection {
    pub user: User,
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


pub async fn create() -> ResContext<Vec<Connection>> {
    let Opts { users_file, proxy, proxies_file, host, count, .. } = Opts::get();

    let users = {
        let file = File::open(&users_file).context(|| format!("opening users ({})", users_file))?;
        read_users(file).context(|| format!("reading users ({})", users_file))?
    };

    if users.len() < count {
        err(format!("there are {} users but {} were requested", users.len(), count))?
    }

    let users = &users[..count];

    let host = format!("{}:{}", host, 25565);

    let connections = obtain_connections(proxy, &proxies_file, &host, users).await?;

    Ok(connections)
}
