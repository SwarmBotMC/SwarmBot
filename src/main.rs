//! The entry point for the code

#[macro_use]
extern crate swarm_bot_packets;

use std::pin::Pin;

use anyhow::Context;
use futures::Stream;
use tokio::{runtime::Runtime, task};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{
    bootstrap::{
        dns::normalize_address, opts::CliOptions, storage::BotConnectionData, BotConnection,
    },
    client::runner::{Runner, RunnerOptions},
};

mod bootstrap;
mod client;
mod protocol;
mod schematic;
mod storage;
mod types;

fn main() {
    // create the single-threaded async runtime
    // we still leverage threads—however in a non-async context.
    // For instance, A* and other CPU-heavy tasks are spawned into threads
    let rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();
    local.block_on(&rt, async move {
        match run().await {
            // this should never happen as this should be an infinite loop
            Ok(()) => println!("Program exited without errors somehow"),

            // print the error in non-debug fashion
            Err(err) => println!("{err:?}"),
        }
    });
}

fn default<T: Default>() -> T {
    T::default()
}

async fn run() -> anyhow::Result<()> {
    // grab options from CLI
    let CliOptions {
        users_file,
        proxies_file,
        host,
        count,
        ver: version,
        port,
        delay_ms,
        ws_port,
        proxy,
        online,
    } = CliOptions::get();

    // A list of users we will login

    // looks up DNS records, etc. This is important where there is a redirect
    // for instance, 2b2t.org has a DNS redirect
    let server_address = normalize_address(&host, port).await;

    let connection_data: Pin<Box<dyn Stream<Item = BotConnectionData>>> = match online {
        false => Box::pin(BotConnectionData::offline_random().take(count)),
        true => {
            let bot_receiver =
                BotConnectionData::load_from_files(&users_file, &proxies_file, proxy, count)?;

            Box::pin(ReceiverStream::new(bot_receiver))
        }
    };

    // taking the users and generating connections to the Minecraft server
    let connections: ReceiverStream<_> =
        BotConnection::stream(server_address, connection_data).into();

    // only return bot connections which were successful
    let connections = connections.filter_map(|elem| match elem {
        Ok(v) => Some(v),
        Err(e) => {
            println!("was unable to create a connection for a user: {e}");
            None
        }
    });

    let run_options = RunnerOptions { delay_ms, ws_port };

    // launch the runner with the appropriate protocol version
    match version {
        340 => Runner::<protocol::v340::Protocol>::run(connections, run_options)
            .await
            .context("Error starting up 1.12")?, // 1.12.2
        _ => {
            panic!("version {version} does not exist")
        }
    }

    Ok(())
}
