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
#![allow(dead_code)]
#![allow(incomplete_features)]
#![deny(unused_must_use)]
#![deny(warnings)]
#![deny(rustdoc::broken_intra_doc_links)]
// #![deny(clippy::panic)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::await_holding_refcell_ref)]
#![deny(clippy::use_debug)]
#![feature(in_band_lifetimes)]
#![feature(generic_const_exprs)]
#![feature(path_try_exists)]
#![feature(type_alias_impl_trait)]
#![feature(once_cell)]
#![feature(step_trait)]
#![feature(option_get_or_insert_default)]
#![feature(array_zip)]
#![feature(test)]
#![feature(box_syntax)]
#![feature(default_free_fn)]
#![feature(bool_to_option)]

#[macro_use]
extern crate enum_dispatch;
extern crate serde;
#[macro_use]
extern crate swarm_bot_packets;
extern crate test;
#[macro_use]
extern crate thiserror;

use tokio::{runtime::Runtime, task};

use crate::{
    bootstrap::{dns::normalize_address, opts::Opts, storage::BotData, Connection},
    client::runner::{Runner, RunnerOptions},
    error::{HasContext, ResContext},
};

mod bootstrap;
mod client;
mod error;
mod protocol;
mod schematic;
mod storage;
mod term;
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
            Err(err) => println!("{}", err),
        }
    });
}

async fn run() -> ResContext {
    let Opts {
        users_file,
        proxies_file,
        host,
        count,
        version,
        port,
        delay_ms,
        load,
        ws_port,
        proxy,
    } = Opts::get();

    // A list of users we will login
    let mut bot_receiver = BotData::load(proxy, &users_file, &proxies_file, count)?;

    if load {
        while bot_receiver.recv().await.is_some() {
            // empty
        }
        return Ok(());
    }

    // looks up DNS records, etc
    let server_address = normalize_address(&host, port).await;

    // taking the users and generating connections to the Minecraft server
    let connections = Connection::stream(server_address, bot_receiver);

    let run_options = RunnerOptions { delay_ms, ws_port };

    match version {
        340 => Runner::<protocol::v340::Protocol>::run(connections, run_options)
            .await
            .context_str("Error starting up 1.12")?, // 1.12
        _ => {
            panic!("version {} does not exist", version)
        }
    }

    Ok(())
}
