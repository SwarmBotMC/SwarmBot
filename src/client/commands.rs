use std::sync::mpsc::Receiver;

use futures::StreamExt;
use interfaces::CommandData;
use serde_json::Value;
use tokio::net::TcpListener;

/// commands received over websocket (typically forge mod)
pub struct CommandReceiver {
    pub pending: Receiver<CommandData>,
}

fn process(path: &str, value: Value) -> Option<CommandData> {
    macro_rules! parse {
        () => {{
            serde_json::from_value(value).unwrap()
        }};
    }

    match path {
        "mine" => Some(CommandData::Mine(parse!())),
        "goto" => Some(CommandData::GoTo(parse!())),
        "attack" => Some(CommandData::Attack(parse!())),

        path => {
            println!("invalid {path}");
            None
        }
    }
}

impl CommandReceiver {
    pub async fn init(port: u16) -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let server = TcpListener::bind(format!("127.0.0.1:{port}")).await?;

        tokio::task::spawn_local(async move {
            loop {
                let (stream, _) = server.accept().await.unwrap();
                let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();

                let tx = tx.clone();

                tokio::task::spawn_local(async move {
                    'wloop: while let Some(msg) = ws.next().await {
                        let msg = msg.unwrap();

                        let text = msg.into_text().unwrap();

                        let mut v: Value = match serde_json::from_str(&text) {
                            Ok(v) => v,
                            Err(_e) => continue 'wloop,
                        };

                        let Value::Object(map) = &mut v else { panic!("invalid value") };

                        let Value::String(path) = map.remove("path").expect("no path elem") else { panic!("invalid path") };

                        let command = process(&path, v).expect("invalid command");
                        tx.send(command).unwrap();
                    }
                });
            }
        });

        Ok(Self { pending: rx })
    }
}
