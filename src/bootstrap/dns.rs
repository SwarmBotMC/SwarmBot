//! Module for interacting with DNS
use anyhow::Context;
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    AsyncResolver,
};

use crate::bootstrap::Address;

/// performs a DNS lookup on host
async fn dns_lookup(host: &str) -> anyhow::Result<Address> {
    let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

    let res = resolver
        .srv_lookup(format!("_minecraft._tcp.{host}"))
        .await
        .context("could not perform SRV lookup")?;

    let srv = res
        .iter()
        .next()
        .context("there are no elements in SRV lookup")?;

    Ok(Address {
        host: srv.target().to_utf8(),
        port: srv.port(),
    })
}

/// Normalizes the address. Some servers like 2b2t have a separate address for
/// minecraft since they have a special mc record
pub async fn normalize_address(host: &str, port: u16) -> Address {
    match dns_lookup(host).await {
        Ok(res) => res,
        Err(_) => Address {
            host: host.to_string(),
            port,
        },
    }
}
