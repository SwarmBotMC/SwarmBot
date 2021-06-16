#![allow(unused)]
#![deny(unused_must_use)]
#![feature(async_closure)]
#![feature(never_type)]
#![feature(in_band_lifetimes)]
#![feature(drain_filter)]

use tokio::runtime::Runtime;
use tokio::task;

use crate::bootstrap::Output;
use crate::client::runner::{Runner, RunnerOptions};
use crate::error::ResContext;

mod error;
mod bootstrap;
mod protocol;
mod client;
mod storage;
mod db;
mod types;


fn main() {
    match run() {
        Ok(_) => println!("Program exited without errors somehow"),
        Err(err) => println!("{}", err)
    };
}

fn run() -> ResContext<()> {
    let mut rt = Runtime::new().unwrap();
    let local = task::LocalSet::new();

    local.block_on(&rt, async move {
        let Output { version, delay_millis, connections } = bootstrap::init().await?;
        let opts = RunnerOptions {delay_millis};

        match version {
            340 => Runner::<protocol::v340::Protocol>::run(connections, opts).await,
            _ => { panic!("version {} does not exist", version) }
        }

        Ok(())
    })
}
