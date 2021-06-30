/*
 * Copyright (c) 2021 Minecraft IGN RevolutionNow - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by RevolutionNow <Xy8I7.Kn1RzH0@gmail.com>, 6/29/21, 8:16 PM
 */

#![allow(dead_code)]
#![deny(unused_must_use)]
#![feature(in_band_lifetimes)]
#![feature(path_try_exists)]
#![feature(const_evaluatable_checked)]
#![feature(const_generics)]
#![feature(once_cell)]
#![feature(step_trait)]
#![feature(option_get_or_insert_default)]
#![feature(array_zip)]
#![feature(box_syntax)]
#![feature(default_free_fn)]

use std::fs::File;

use tokio::runtime::Runtime;
use tokio::task;

use crate::bootstrap::blocks::BlockData;
use crate::bootstrap::Connection;
use crate::bootstrap::dns::normalize_address;
use crate::bootstrap::opts::Opts;
use crate::bootstrap::storage::UserCache;
use crate::client::runner::{Runner, RunnerOptions};
use crate::error::{HasContext, ResContext};

mod error;
mod bootstrap;
mod protocol;
mod client;
mod storage;
mod types;

fn main() {
    let rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();
    local.block_on(&rt, async move {
        match run().await {
            Ok(_) => println!("Program exited without errors somehow"),
            Err(err) => println!("{}", err)
        }
    });
}


async fn run() -> ResContext<()> {
    let Opts { users_file, proxy: _, proxies_file, host, count, version, port, db: _, delay, load, .. } = Opts::get();

    let address = normalize_address(&host, port).await;

    // A list of users we will login
    let mut proxy_users = {
        println!("reading {}", users_file);
        let csv_file = File::open(&users_file).context(|| format!("could not open users file {}", users_file))?;
        let csv_users = bootstrap::csv::read_users(csv_file).context_str("could not open users file")?;

        println!("reading {}", proxies_file);
        let proxies_file = File::open(&proxies_file).context(|| format!("could not open proxies file {}", proxies_file))?;
        let proxies = bootstrap::csv::read_proxies(proxies_file).context_str("could not open proxies file")?;

        println!("reading cache.db");
        let cache = UserCache::load("cache.db".into());

        println!("obtaining users from cache");
        cache.obtain_users(count, csv_users, proxies)
    };

    let blocks = BlockData::read().context_str("error reading blocks file")?;

    if load {
        while proxy_users.recv().await.is_some() {
            // empty
        }
        return Ok(());
    } else {
        // taking the users and generating connections to the Minecraft server
        let connections = Connection::stream(address, proxy_users);

        let opts = RunnerOptions { delay_millis: delay, blocks };

        match version {
            340 => Runner::<protocol::v340::Protocol>::run(connections, opts).await, // 1.12
            _ => { panic!("version {} does not exist", version) }
        }
    }


    Ok(())
}
