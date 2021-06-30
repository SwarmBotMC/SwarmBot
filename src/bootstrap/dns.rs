/*
 * Copyright (c) 2021 Minecraft IGN RevolutionNow - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by RevolutionNow <Xy8I7.Kn1RzH0@gmail.com>, 6/29/21, 8:16 PM
 */

use trust_dns_resolver::AsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveError;

use crate::bootstrap::Address;

async fn dns_lookup(host: &str) -> Result<Address, ResolveError> {
    let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    println!("performing srv lookup");
    resolver.srv_lookup(format!("_minecraft._tcp.{}", host)).await.map(|res| {
        let srv = res.iter().next().unwrap();
        Address {
            host: srv.target().to_utf8(),
            port: srv.port(),
        }
    })
}


pub async fn normalize_address(host: &str, port: u16) -> Address {
    match dns_lookup(host).await {
        Ok(res) => res,
        Err(_) => {
            Address {
                host: host.to_string(),
                port,
            }
        }
    }
}
