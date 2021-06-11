use std::fs::File;


use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

use crate::error::{HasContext, ResContext};
use crate::connections::csv::read_proxies;
use crate::connections::{User, Connection};

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
