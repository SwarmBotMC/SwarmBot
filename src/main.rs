#![allow(dead_code)]
#![allow(incomplete_features)]
#![deny(unused_must_use)]
#![deny(warnings)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::await_holding_refcell_ref)]
#![feature(generic_const_exprs)]
#![feature(type_alias_impl_trait)]
#![feature(once_cell)]
#![feature(step_trait)]
#![feature(option_get_or_insert_default)]
#![feature(array_zip)]
#![feature(test)]
#![feature(box_syntax)]
#![feature(default_free_fn)]
#![feature(fs_try_exists)]

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
    bootstrap::{dns::normalize_address, opts::CliOptions, storage::BotDataLoader, BotConnection},
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
    // multiple threads will still be used—however, they will only be
    // used in non-async context as they are resource heavy
    let rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();
    local.block_on(&rt, async move {
        match run().await {
            // this should never happen as this should be an infinite loop
            Ok(_) => println!("Program exited without errors somehow"),

            // print the error in non-debug fashion
            Err(err) => println!("{err}"),
        }
    });
}

async fn run() -> ResContext {
    // grab options from CLI
    let CliOptions {
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
    } = CliOptions::get();

    // A list of users we will login
    let mut bot_receiver = BotDataLoader::load(proxy, &users_file, &proxies_file, count)?;

    // if we only load the data but do not login
    if load {
        while bot_receiver.recv().await.is_some() {
            // empty
        }
        return Ok(());
    }

    // looks up DNS records, etc. This is important where there is a redirect
    // for instance, 2b2t.org has a DNS redirect
    let server_address = normalize_address(&host, port).await;

    // taking the users and generating connections to the Minecraft server
    let connections = BotConnection::stream(server_address, bot_receiver);

    let run_options = RunnerOptions { delay_ms, ws_port };

    // launch the runner with the appropriate protocol version
    match version {
        340 => Runner::<protocol::v340::Protocol>::run(connections, run_options)
            .await
            .context_str("Error starting up 1.12")?, // 1.12
        _ => {
            panic!("version {version} does not exist")
        }
    }

    Ok(())
}
