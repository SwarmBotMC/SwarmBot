use std::fs::File;

use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

use crate::bootstrap::{Connection, User};
use crate::bootstrap::csv::read_proxies;
use crate::bootstrap::mojang::Mojang;
use crate::error::{HasContext, ResContext};

pub async fn obtain_connections(proxy: bool, proxies: &str, host: &str, port: u16, users: &[User]) -> ResContext<Vec<Connection>> {
    let addr = format!("{}:{}", host, port);

    let count = users.len();
    let streams = {
        let mut inner = Vec::with_capacity(count);

        match proxy {
            false => {
                let stream = TcpStream::connect(&addr).await.context(|| format!("connecting to server"))?;
                let mojang = Mojang::default();
                inner.push((stream, mojang))
            }
            true => {
                let file = File::open(proxies).context(|| format!("opening proxy ({})", proxies))?;
                let proxies = read_proxies(file).context(|| format!("opening proxies ({})", proxies))?;

                for proxy in proxies.iter().cycle().take(count) {
                    let addr = proxy.address();
                    let stream = Socks5Stream::connect_with_password(addr.as_str(), addr.as_str(), &proxy.user, &proxy.pass);
                    let stream = stream.await.context(|| format!("connecting to proxy {}", proxy.address()))?;

                    let mojang = Mojang::socks5(&addr, &proxy.user, &proxy.pass).context(||format!("generating mojang https client"))?;
                    inner.push((stream.into_inner(), mojang))
                }
            }
        }

        inner
    };

    Ok(users.iter().zip(streams).map(|(user, (stream, mojang))| {
        let online = user.online;
        let user = user.clone();
        let (read, write) = stream.into_split();
        Connection {
            mojang,
            user,
            online,
            host: host.to_string(),
            read,
            write,
            port,
        }
    }).collect())
}
