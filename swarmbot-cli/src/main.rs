#![feature(never_type)]

use std::{io, io::Write};

use anyhow::Context;
use clap::Parser;
use futures::SinkExt;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpSocket,
};
use tokio_tungstenite::tungstenite::Message;

/// Options parsed from CLI
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Andrew Gazelka")]
struct CliOptions {
    /// The IP we are going to connect to
    #[clap(long, default_value = "127.0.0.1")]
    pub ip: String,

    /// The port of the web socket that is used to communicate bot commands
    /// to. This is used to interface with the SwarmBot mod, although it
    /// can be used for anything.
    #[clap(long, default_value = "8080")]
    pub port: u16,
}

async fn run() -> anyhow::Result<!> {
    let CliOptions { ip, port } = CliOptions::parse();

    let addr = format!("{ip}:{port}");
    let addr = addr.parse()?;

    let socket = TcpSocket::new_v4().context("could not create V4 socket")?;
    let socket = socket
        .connect(addr)
        .await
        .context("could not connect to given address")?;

    let mut web_socket = tokio_tungstenite::accept_async(socket)
        .await
        .context("could not create websocket")?;

    println!();
    let mut stdin = BufReader::new(tokio::io::stdin());

    let mut s = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;

        let len = stdin.read_line(&mut s).await?;
        let s = &s[..len];

        web_socket.send(Message::Text(s.to_string())).await?;
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        println!("got error {err}")
    }
}
