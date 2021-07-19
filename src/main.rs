/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
#![allow(dead_code)]
#![allow(incomplete_features)]
#![deny(unused_must_use)]
#![feature(impl_trait_in_bindings)]
#![feature(in_band_lifetimes)]
#![feature(path_try_exists)]
#![feature(const_evaluatable_checked)]
#![feature(const_generics)]
#![feature(min_type_alias_impl_trait)]
#![feature(once_cell)]
#![feature(step_trait)]
#![feature(option_get_or_insert_default)]
#![feature(array_zip)]
#![feature(test)]
#![feature(box_syntax)]
#![feature(default_free_fn)]

#[macro_use]
extern crate enum_dispatch;
extern crate serde;
#[macro_use]
extern crate swarm_bot_packets;
extern crate test;
#[macro_use]
extern crate thiserror;

use std::fs::File;

use tokio::runtime::Runtime;
use tokio::task;

use crate::bootstrap::Connection;
use crate::bootstrap::dns::normalize_address;
use crate::bootstrap::opts::Opts;
use crate::bootstrap::storage::UserCache;
use crate::client::runner::{Runner, RunnerOptions};
use crate::error::{HasContext, ResContext};


mod error;
mod bootstrap;
mod protocol;
mod term;
mod client;
mod storage;
mod schematic;
mod types;

fn main() {

    // create the single-threaded async runtime
    let rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();
    local.block_on(&rt, async move {
        match run().await {
            // this should never happen as this should be an infinite loop
            Ok(_) => println!("Program exited without errors somehow"),

            // print the error in non-debug fashion
            Err(err) => println!("{}", err)
        }
    });
}


async fn run() -> ResContext {
    let Opts { users_file, proxies_file, host, count, version, port, delay, load } = Opts::get();

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

    if load {
        while proxy_users.recv().await.is_some() {
            // empty
        }
        return Ok(());
    } else {
        // taking the users and generating connections to the Minecraft server
        let connections = Connection::stream(address, proxy_users);

        let opts = RunnerOptions { delay_millis: delay };

        match version {
            340 => Runner::<protocol::v340::Protocol>::run(connections, opts).await.context_str("Error starting up 1.12")?, // 1.12
            _ => { panic!("version {} does not exist", version) }
        }
    }


    Ok(())
}
