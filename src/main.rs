//! The entry point for the code

#![feature(generic_const_exprs)]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![feature(step_trait)]
#![feature(option_get_or_insert_default)]
#![feature(test)]
#![feature(default_free_fn)]
#![feature(fs_try_exists)]
#![feature(async_fn_in_trait)]
#![deny(
    clippy::await_holding_refcell_ref,
    clippy::await_holding_lock,
    rustdoc::broken_intra_doc_links,
    unused_must_use,
    unused_extern_crates,
    warnings,
    clippy::complexity,
    clippy::correctness,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
    clippy::expect_used,
    clippy::panic,
    clippy::rc_buffer,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::undocumented_unsafe_blocks,
    clippy::unneeded_field_pattern,
    clippy::unwrap_used,
    clippy::verbose_file_reads,
    clippy::negative_feature_names,
    clippy::redundant_feature_names,
    clippy::wildcard_dependencies,
    clippy::iter_with_drain,
    clippy::missing_const_for_fn,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::nonstandard_macro_braces,
    clippy::path_buf_push_overwrite,
    clippy::redundant_pub_crate,
    clippy::suspicious_operation_groupings,
    clippy::use_self,
    clippy::useless_let_if_seq
)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_sign_loss)]
// TODO: remove most of these
#![allow(
    incomplete_features,
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::default_trait_access,
    clippy::match_bool
)]

// TODO: uncomment these
// #![deny(missing_docs)]
// #![deny(clippy::missing_docs_in_private_items)]

#[allow(unused, clippy::useless_attribute)]
extern crate test;

#[macro_use]
extern crate enum_dispatch;
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
            Ok(_) => println!("Program exited without errors somehow"),

            // print the error in non-debug fashion
            Err(err) => println!("{err}"),
        }
    });
}

async fn run() -> anyhow::Result<()> {
    // grab options from CLI
    let CliOptions {
        users_file,
        proxies_file,
        host,
        count,
        version,
        port,
        delay_ms,
        ws_port,
        proxy,
        offline,
    } = CliOptions::get();

    // A list of users we will login

    // looks up DNS records, etc. This is important where there is a redirect
    // for instance, 2b2t.org has a DNS redirect
    let server_address = normalize_address(&host, port).await;

    let connection_data: Pin<Box<dyn Stream<Item = BotConnectionData>>> = match offline {
        true => Box::pin(BotConnectionData::offline_random().take(count)),
        false => {
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
