use std::fs::File;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

use crate::{User};
use crate::csv::read_proxies;
use crate::error::{HasContext, ResContext};

#[derive(Debug)]
pub struct Connection {
    pub user: User,
    pub read: OwnedReadHalf,
    pub write: OwnedWriteHalf,
}

pub async fn obtain_connections(proxy: bool, proxies: &str, host: &str, users: &[User]) -> ResContext<Vec<Connection>> {
    let count = users.len();
    let streams = {
        let mut inner = Vec::with_capacity(count);

        match proxy {
            false => {
                let stream = TcpStream::connect(&host).await.context(|| format!("connecting to server"))?;
                inner.push(stream)
            }
            true => {
                let file = File::open(proxies).context(|| format!("opening proxy ({})", proxies))?;
                let proxies = read_proxies(file).context(|| format!("opening proxies ({})", proxies))?;

                for proxy in proxies.iter().cycle().take(count) {
                    let addr = proxy.address();
                    let stream = Socks5Stream::connect_with_password(addr.as_str(), host, &proxy.user, &proxy.pass);
                    let stream = stream.await.context(|| format!("connecting to proxy {}", proxy.address()))?;
                    inner.push(stream.into_inner())
                }
            }
        }

        inner
    };

    Ok(users.iter().zip(streams).map(|(user, stream)| {
        let user = user.clone();
        let (read, write) = stream.into_split();
        Connection {
            user,
            read,
            write,
        }
    }).collect())
}
