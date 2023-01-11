#![feature(once_cell)]

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tungstenite::Message;

use crate::types::{BlockLocation, Selection2D};

pub mod types;

type Id = u64;

/// The mine command.
/// Mine the given selection.
/// A global command. The process should allocate appropriately to children.
#[derive(Serialize, Deserialize, Debug)]
pub struct Mine {
    pub sel: Selection2D,
}

/// A navigation command to go to the given block location
#[derive(Serialize, Deserialize, Debug)]
pub struct GoTo {
    pub location: BlockLocation,
}

/// Attack a given player
#[derive(Serialize, Deserialize, Debug)]
pub struct Attack {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cancelled {
    pub id: Id,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Finished {
    pub id: Id,
}

macro_rules! commands {
    (
        $($command: ident),*
    ) =>
    {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "lowercase")]
        #[serde(tag = "type")]
        pub enum CommandData {
            $($command($command)),*
        }

    };
}

commands! {
    Mine, GoTo, Attack, Cancelled, Finished
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    id: u64,
    data: CommandData,
}

pub struct Comm {
    rx: tokio::sync::mpsc::UnboundedReceiver<Command>,
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
}

type Res<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn incoming(msg: tungstenite::Message) -> Res<Command> {
    let data = msg.into_data();
    let command: Command = serde_json::from_slice(&data)?;
    Ok(command)
}

fn outgoing(command: Command) -> Res<Message> {
    let string = serde_json::to_string(&command)?;
    Ok(Message::Text(string))
}

impl Comm {
    pub async fn recv(&mut self) -> Command {
        self.rx.recv().await.unwrap()
    }

    pub fn send(&mut self, command: Command) {
        self.tx.send(command).unwrap();
    }

    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Res<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (recv_tx, recv_rx) = tokio::sync::mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut ws = tokio_tungstenite::accept_async(stream).await?;
        tokio::spawn(async move {
            let mut msg_to_send = None;
            tokio::select! {
                val = ws.next() => {
                    if let Some(Ok(msg)) = val {
                       if let Ok(cmd) = incoming(msg)  {
                            let _ = recv_tx.send(cmd);
                        }
                    }
                }
                val = send_rx.recv() => {
                    if let Some(cmd) = val {
                        if let Ok(msg) = outgoing(cmd) {
                            msg_to_send = Some(msg);
                        }
                    }
                }
            }

            if let Some(msg) = msg_to_send {
                let _ = ws.send(msg).await;
            }
        });

        Ok(Self {
            rx: recv_rx,
            tx: send_tx,
        })
    }

    pub async fn host<A: ToSocketAddrs>(addr: A) -> Res<Self> {
        let server = TcpListener::bind(addr).await?;
        let (recv_tx, recv_rx) = tokio::sync::mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let _ignored: Res = async {
                    let (stream, _) = server.accept().await?;
                    let mut ws = tokio_tungstenite::accept_async(stream).await?;

                    let mut msg_to_send = None;
                    tokio::select! {
                        val = ws.next() => {
                            if let Some(Ok(msg)) = val {
                               if let Ok(cmd) = incoming(msg)  {
                                    let _ = recv_tx.send(cmd);
                                }
                            }
                        }
                        val = send_rx.recv() => {
                            if let Some(cmd) = val {
                                if let Ok(msg) = outgoing(cmd) {
                                    msg_to_send = Some(msg)
                                }
                            }
                        }
                    }

                    if let Some(msg) = msg_to_send {
                        let _ = ws.send(msg).await;
                    }

                    Ok(())
                }
                .await;
            }
        });

        Ok(Self {
            rx: recv_rx,
            tx: send_tx,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Attack, Command, CommandData};

    #[test]
    fn test() {
        let command = Command {
            id: 123,
            data: CommandData::Attack(Attack {
                name: "hello".to_string(),
            }),
        };

        serde_json::to_string(&command).unwrap();
    }
}
