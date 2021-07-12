/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/8/21, 8:37 PM
 */


use std::convert::TryFrom;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::{TcpListener, TcpSocket};
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::Message;

use crate::error::Res;
use crate::storage::block::{BlockLocation2D, BlockLocation};

pub struct Commands {
    pub pending: Receiver<Command>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Selection2D {
    pub from: BlockLocation2D,
    pub to: BlockLocation2D,
}

impl Selection2D {
    pub fn normalize(self) -> Self {
        let min_x = self.from.x.min(self.to.x);
        let min_z = self.from.z.min(self.to.z);

        let max_x = self.from.x.max(self.to.x);
        let max_z = self.from.z.max(self.to.z);

        Selection2D {
            from: BlockLocation2D::new(min_x, min_z),
            to: BlockLocation2D::new(max_x, max_z),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mine {
    pub sel: Selection2D,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct GoTo {
    pub location: BlockLocation
}

pub enum Command {
    Mine(Mine),
    GoTo(GoTo)
}


fn process(path: &str, value: Value) -> Option<Command> {
    match path {
        "mine" => {
            let mine = serde_json::from_value(value).unwrap();
            Some(Command::Mine(mine))
        }
        "goto" => {
            let goto = serde_json::from_value(value).unwrap();
            Some(Command::GoTo(goto))
        }
        path => {
            println!("invalid {}", path);
            None
        }
    }
}


impl Commands {
    pub async fn init() -> Res<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let server = TcpListener::bind("127.0.0.1:8080").await?;

        tokio::task::spawn_local(async move {
            loop {
                let (stream, _) = server.accept().await.unwrap();
                let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();

                let tx = tx.clone();

                tokio::task::spawn_local(async move {
                    'wloop:
                    while let Some(msg) = ws.next().await {
                        let msg = msg.unwrap();

                        let text = msg.into_text().unwrap();

                        let mut v: Value = match serde_json::from_str(&text) {
                            Ok(v) => v,
                            Err(e) => continue 'wloop,
                        };

                        let mut map = match &mut v {
                            Value::Object(map) => map,
                            _ => panic!("invalid value"),
                        };

                        let path = match map.remove("path").expect("no path elem") {
                            Value::String(path) => path,
                            _ => panic!("invalid path")
                        };

                        let command = process(&path, v).expect("invalid command");
                        tx.send(command).unwrap();
                    }
                });
            }
        });

        Ok(Self {
            pending: rx
        })
    }
}
