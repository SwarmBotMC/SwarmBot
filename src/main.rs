//! The entry point for the code

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
#![feature(async_fn_in_trait)]
// #![deny(missing_docs)]
// #![deny(clippy::missing_docs_in_private_items)]

// #![deny(clippy::indexing_slicing)]

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
#![allow(incomplete_features, clippy::items_after_statements, clippy::cast_possible_truncation, clippy::module_name_repetitions, clippy::unwrap_used, clippy::indexing_slicing, clippy::panic, clippy::expect_used, clippy::cast_precision_loss, clippy::cast_possible_wrap, clippy::default_trait_access)]

#[allow(unused, clippy::useless_attribute)]
extern crate test;

#[macro_use]
extern crate enum_dispatch;
#[macro_use]
extern crate swarm_bot_packets;

use anyhow::Context;
use tokio::{runtime::Runtime, task};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{
    bootstrap::{dns::normalize_address, opts::CliOptions, storage::BotDataLoader, BotConnection},
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
    let connections: ReceiverStream<_> = BotConnection::stream(server_address, bot_receiver).into();

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
            .context("Error starting up 1.12")?, // 1.12
        _ => {
            panic!("version {version} does not exist")
        }
    }

    Ok(())
}
