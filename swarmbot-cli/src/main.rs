#![feature(never_type)]

use std::{io, io::Write};

use anyhow::{bail, Context};
use clap::Parser;
use futures::SinkExt;
use swarmbot_interfaces::{types::BlockLocation, CommandData, GoTo};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::tungstenite::Message;

/// Options parsed from CLI
///
/// Commands:
///
///
/// goto {x} {y} {z}    — go to the coordinates {x} {y} {z}
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

    let (mut web_socket, _) = tokio_tungstenite::connect_async(format!("ws://{ip}:{port}"))
        .await
        .context("could not create websocket")?;

    println!("connected to websocket");
    println!();
    let stdin = BufReader::new(tokio::io::stdin());

    let mut lines = stdin.lines();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let s = lines.next_line().await?.unwrap_or_default();

        match send_string(&s) {
            Ok(s) => web_socket.send(Message::Text(s)).await?,
            Err(e) => println!("invalid... {e}"),
        }
    }
}

fn send_string(input: &str) -> anyhow::Result<String> {
    let cmd_data = to_command_data(input)
        .with_context(|| format!("invalid converting to command data for {input}"))?;
    let to_send = serde_json::to_string(&cmd_data).context("converting to JSON")?;

    println!("sending {to_send}");
    println!();
    Ok(to_send)
}

fn to_command_data(input_str: &str) -> anyhow::Result<CommandData> {
    let mut input = input_str.trim().split(' ');

    let cmd_name = input.next().context("no command name specified")?;

    match cmd_name {
        "goto" => Ok(CommandData::GoTo(GoTo {
            location: BlockLocation {
                x: input.next().context("no x in goto")?.parse()?,
                y: input.next().context("no y in goto")?.parse()?,
                z: input.next().context("no z in goto")?.parse()?,
            },
        })),
        _ => bail!("input '{input_str}' could not be parsed into a CommandData struct"),
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        println!("got error {err}")
    }
}
