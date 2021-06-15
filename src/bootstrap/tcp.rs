use std::fs::File;

use itertools::Itertools;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

use crate::bootstrap::{Connection, User};
use crate::bootstrap::csv::read_proxies;
use crate::bootstrap::mojang::Mojang;
use crate::error::{HasContext, ResContext};
use rand::seq::SliceRandom;
use crate::db::{Db, CachedUser};

pub async fn obtain_connections(proxy: bool, proxies: &str, host: &str, port: u16, user_count: usize, db: &Db) -> ResContext<tokio::sync::mpsc::UnboundedReceiver<Connection>> {

    let host = String::from(host);
    let users = db.obtain_users(user_count).await;

    let addr = format!("{}:{}", host, port);

    let count = users.len();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    match proxy {
        false => {
            tokio::task::spawn_local(async move {
                for user in users {
                    let stream = TcpStream::connect(&addr).await.context(|| format!("connecting to server")).unwrap();
                    let mojang = Mojang::default();
                    tx.send(combine(user, stream, mojang, host.clone(), port)).unwrap();
                }
            });
        }
        true => {
            let file = File::open(proxies).context(|| format!("opening proxy ({})", proxies))?;
            let mut proxies = read_proxies(file).context(|| format!("opening proxies ({})", proxies))?;

            tokio::task::spawn_local(async move {
                for (proxy, user) in proxies.iter().cycle().zip(users) {

                    let proxy_addr = proxy.address();
                    let actual_addr = format!("{}:{}", host, port);

                    let stream = Socks5Stream::connect_with_password(proxy_addr.as_str(), actual_addr.as_str(), &proxy.user, &proxy.pass).await.unwrap();
                    let stream = stream.into_inner();

                    let mojang = Mojang::default();
                    // let mojang = Mojang::socks5(proxy_addr.as_str(), &proxy.user, &proxy.pass).context(|| format!("generating mojang https client")).unwrap();

                    tx.send(combine(user, stream, mojang, host.clone(), port)).unwrap();
                }
            });
        }
    }

    Ok(rx)
}

fn combine(user: CachedUser, stream: TcpStream, mojang: Mojang, host: String, port: u16) -> Connection {
    let (read, write) = stream.into_split();
    Connection {
        user,
        mojang,
        host,
        read,
        write,
        port,
    }
}
