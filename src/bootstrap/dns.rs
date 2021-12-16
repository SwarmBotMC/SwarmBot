// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    error::ResolveError,
    AsyncResolver,
};

use crate::bootstrap::Address;

async fn dns_lookup(host: &str) -> Result<Address, ResolveError> {
    let resolver =
        AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    println!("performing srv lookup");
    resolver
        .srv_lookup(format!("_minecraft._tcp.{}", host))
        .await
        .map(|res| {
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
        Err(_) => Address {
            host: host.to_string(),
            port,
        },
    }
}
