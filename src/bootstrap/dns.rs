use trust_dns_resolver::AsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveError;

use crate::bootstrap::Address;
use crate::error::Res;

pub async fn dns_lookup(host: &str) -> Result<Address, ResolveError> {
    let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    resolver.srv_lookup(format!("_minecraft._tcp.{}", host)).await.map(|res| {
        let srv = res.iter().next().unwrap();
        Address {
            host: srv.target().to_utf8(),
            port: srv.port(),
        }
    })
}
