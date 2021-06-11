#![feature(async_closure)]
#![feature(never_type)]

use crate::client::runner::Runner;
use crate::error::ResContext;

mod error;
mod connections;
mod launcher;
mod protocol;
mod client;


#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Program exited without errors somehow"),
        Err(err) => println!("{}", err)
    };
}

async fn run() -> ResContext<!> {
    let connections = connections::create().await?;
    let conns = connections.list;

    match connections.version {
        340 => Runner::<protocol::v340::Protocol>::run(conns).await,
        _ => { panic!("version {} does not exist", connections.version) }
    }
}
