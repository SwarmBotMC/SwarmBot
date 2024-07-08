use std::sync::mpsc::{Receiver, Sender};

use anyhow::{bail, Context};
use futures::StreamExt;
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tracing::error;

/// commands received over websocket (typically forge mod)
pub struct CommandReceiver {
    pub pending: Receiver<TaggedValue>,
}

struct Processor {
    value: Value,
}

pub struct TaggedValue {
    pub path: String,
    pub value: Value,
}

impl TaggedValue {
    pub fn parse<T>(self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.value)
    }
}

async fn command_receiver(
    tx: Sender<TaggedValue>,
    mut ws: WebSocketStream<TcpStream>,
) -> anyhow::Result<()> {
    while let Some(msg) = ws.next().await {
        let msg = msg.context("error reading next web socket message (websocket disconnect?)")?;

        let text = msg.into_text().unwrap();

        let v: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                error!("invalid json {e}");
                continue;
            }
        };

        let Value::Object(mut map) = v else {
            bail!("expected object")
        };

        let Value::String(path) = map.remove("path").expect("no path elem") else {
            bail!("invalid path")
        };

        let value = Value::Object(map);

        let elem = TaggedValue { path, value };

        tx.send(elem).unwrap();
    }
    Ok(())
}

impl CommandReceiver {
    pub async fn init(port: u16) -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let server = TcpListener::bind(format!("127.0.0.1:{port}")).await?;

        tokio::task::spawn_local(async move {
            loop {
                let (stream, _) = server.accept().await.unwrap();
                let ws = tokio_tungstenite::accept_async(stream).await.unwrap();

                let tx = tx.clone();

                tokio::task::spawn_local(async move {
                    if let Err(e) = command_receiver(tx, ws).await {
                        println!("error with websocket: {e}");
                    }
                });
            }
        });

        Ok(Self { pending: rx })
    }
}
