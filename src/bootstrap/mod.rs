use std::fs::File;

use rand::seq::SliceRandom;
use serde::Deserialize;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::bootstrap::csv::read_users;
use crate::bootstrap::mojang::{AuthResponse, Mojang};
use crate::bootstrap::opts::Opts;
use crate::bootstrap::tcp::obtain_connections;
use crate::db::{Db, ValidDbUser, CachedUser};
use crate::error::{err, Error, HasContext, ResContext};
use std::time::Duration;
use itertools::Itertools;
use packets::types::UUID;
use crate::bootstrap::storage::{ProxyUser, ValidUser};
use tokio_socks::tcp::Socks5Stream;
use tokio::sync::mpsc::Receiver;

pub mod opts;
pub mod csv;
pub mod tcp;
pub mod dns;
pub mod storage;
pub mod mojang;


#[derive(Clone)]
pub struct Address {
    host: String,
    port: u16,
}

impl Into<&String> for Address {
    fn into(self) -> String {
        format!("{}:{}", self.host, self.port)
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
        
        let (tx,rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
           for user in users.recv().await {
               let ProxyUser {proxy, user, mojang} = user.proxy;
               let conn = Socks5Stream::connect_with_password(proxy.address(), (&address).into(), &proxy.user, &proxy.pass).await.unwrap();
               let (read, write) = conn.into_inner().into_split();
               tx.send(Connection {
                   user, address, mojang, read, write
               })
               
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

pub struct Output {
    pub version: usize,
    pub delay_millis: u64,
    pub connections: tokio::sync::mpsc::Receiver<Connection>,
}

