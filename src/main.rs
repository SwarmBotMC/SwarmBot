#![feature(async_closure)]

use std::fs::File;
use serde::Deserialize;
use crate::csv::{read_users};
use crate::error::{err, HasContext, ResContext};
use crate::opts::Opts;
use crate::tcp::obtain_connections;

mod opts;
mod error;
mod csv;
mod tcp;

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


#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Program exited without errors"),
        Err(err) => println!("{}", err)
    };
}

async fn run() -> ResContext {
    let Opts { users_file, proxy, proxies_file, host, count , ..} = Opts::get();

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

    println!("connections {:?}", connections);

    Ok(())
}
