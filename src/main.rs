#![feature(async_closure)]

use crate::error::{ResContext};

mod error;
mod connections;
mod launcher;
mod packet;
mod client;


#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Program exited without errors"),
        Err(err) => println!("{}", err)
    };
}

async fn run() -> ResContext {
    let connections = connections::create().await?;


    Ok(())
}
