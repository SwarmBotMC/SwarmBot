#![allow(unused)]
#![deny(unused_must_use)]
#![feature(async_closure)]
#![feature(never_type)]
#![feature(in_band_lifetimes)]
#![feature(drain_filter)]
#![feature(entry_insert)]
#![feature(path_try_exists)]

use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::task;

use crate::bootstrap::dns::normalize_address;
use crate::bootstrap::mojang::AuthResponse;
use crate::bootstrap::opts::Opts;
use crate::bootstrap::{Output, Connection};
use crate::bootstrap::storage::UserCache;
use crate::client::runner::{Runner, RunnerOptions};
use crate::error::{Error, ResContext, HasContext};
use crate::error::Error::Mojang;

mod error;
mod bootstrap;
mod protocol;
mod client;
mod storage;
mod types;


fn main() {
    let mut rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();
    local.block_on(&rt, async move {
        match run().await {
            Ok(_) => println!("Program exited without errors somehow"),
            Err(err) => println!("{}", err)
        }
    });
}

async fn run() -> ResContext<()> {
    let Opts { users_file, proxy, proxies_file, host, count, version, port, db, delay, .. } = Opts::get();

    let address = normalize_address(&host, port).await;

    // A list of users we will login
    let proxy_users = {
        let csv_file = File::open(&users_file).context(||format!("could not open users file {}", users_file))?;
        let csv_users = bootstrap::csv::read_users(csv_file).context_str("could not open users file")?;

        let proxies_file = File::open(&proxies_file).context(||format!("could not open proxies file {}", proxies_file))?;
        let proxies = bootstrap::csv::read_proxies(proxies_file).context_str("could not open proxies file")?;

        let mut cache = UserCache::load("cache.db".into());
        cache.obtain_users(count, csv_users, proxies)
    };

    // taking the users and generating connections to the Minecraft server
    let connections = Connection::stream(address, proxy_users);

    let opts = RunnerOptions { delay_millis: delay };

    match version {
        340 => Runner::<protocol::v340::Protocol>::run(connections, opts).await,
        _ => { panic!("version {} does not exist", version) }
    }

    Ok(())
}
