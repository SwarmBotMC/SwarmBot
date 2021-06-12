#![feature(async_closure)]
#![feature(never_type)]
#![feature(in_band_lifetimes)]

use crate::bootstrap::Output;
use crate::client::runner::Runner;
use crate::error::ResContext;
use crate::storage::world::WorldBlocks;

mod error;
mod bootstrap;
mod protocol;
mod client;
mod storage;


#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Program exited without errors somehow"),
        Err(err) => println!("{}", err)
    };
}

async fn run() -> ResContext<!> {
    let Output { version, connections } = bootstrap::init().await?;

    let world_blocks = WorldBlocks::default();

    match version {
        340 => Runner::<protocol::v340::Protocol>::run(connections).await,
        _ => { panic!("version {} does not exist", version) }
    }
}
