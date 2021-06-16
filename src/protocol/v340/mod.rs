use std::sync::mpsc::{SendError, TryRecvError};

use packets::types::{PacketState, UUID, VarInt};
use packets::types::Packet;
use tokio::sync::oneshot::error::RecvError;
use tokio::time::{Duration, sleep};

use crate::bootstrap::{Connection, User};
use crate::bootstrap::mojang::{AuthResponse, calc_hash, Mojang};
use crate::client::instance::{ClientInfo, State};
use crate::client::runner::GlobalState;
use crate::db::CachedUser;
use crate::error::Error::WrongPacket;
use crate::error::Res;
use crate::protocol::{Login, McProtocol};
use crate::protocol::encrypt::{rand_bits, RSA};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::{PacketWriteChannel, PacketWriter};
use crate::protocol::types::PacketData;
use crate::protocol::v340::clientbound::{Disconnect, JoinGame, LoginSuccess};
use crate::protocol::v340::serverbound::{HandshakeNextState, TeleportConfirm};
use rand::{thread_rng, Rng};

mod clientbound;
mod serverbound;
mod types;

pub struct Protocol {
    rx: std::sync::mpsc::Receiver<PacketData>,
    tx: PacketWriteChannel,
    disconnected: bool,
}

#[async_trait::async_trait]
impl McProtocol for Protocol {

    async fn login(conn: Connection) -> Res<Login<Self>> {
        let Connection { user, host, port, mojang, read, write } = conn;
        let CachedUser { email, access_token, client_token, username, uuid, password } = user;

        let uuid = UUID::from(&uuid);

        let mut reader = PacketReader::from(read);
        let mut writer = PacketWriter::from(write);

        let online = true;

        // START: handshake
        writer.write(serverbound::Handshake {
            protocol_version: VarInt(340),
            host,
            port,
            next_state: HandshakeNextState::Login,
        }).await;


        // START: login
        writer.write(serverbound::LoginStart {
            username: username.clone()
        }).await;

        // writer.flush().await;

        let clientbound::EncryptionRequest { public_key_der, verify_token, server_id } = reader.read_exact_packet().await?;

        let rsa = RSA::from_der(&public_key_der);

        let shared_secret = rand_bits();

        let encrypted_ss = rsa.encrypt(&shared_secret).unwrap();
        let encrypted_verify = rsa.encrypt(&verify_token).unwrap();

        // Mojang online mode requests
        let hash = calc_hash(&server_id, &shared_secret, &public_key_der);
        mojang.join(uuid, &hash, &access_token).await?;

        // id = 1
        writer.write(serverbound::EncryptionResponse {
            shared_secret: encrypted_ss,
            verify_token: encrypted_verify,
        }).await;

        // writer.flush().await;

        // we now do everything encrypted
        writer.encryption(&shared_secret);
        reader.encryption(&shared_secret);


        // set compression or login success
        {
            let mut data = reader.read().await;

            let LoginSuccess { username, uuid } = match data.id {
                clientbound::SetCompression::ID => {
                    let clientbound::SetCompression { threshold } = data.read();

                    reader.compression(threshold.into());
                    writer.compression(threshold.into());

                    reader.read_exact_packet().await?
                }
                clientbound::LoginSuccess::ID => {
                    data.reader.read()
                }
                // clientbound::Disconnect::ID => {
                //     let clientbound::Disconnect { reason } = data.reader.read();
                //     return Err(Disconnect(reason));
                // }
                actual => {
                    return Err(WrongPacket {
                        state: PacketState::Login,
                        expected: LoginSuccess::ID,
                        actual,
                    });
                }
            };
            // println!("login success for {}", email);
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let (os_tx, os_rx) = tokio::sync::oneshot::channel();

        tokio::task::spawn_local(async move {
            let mut oneshot = Some(os_tx);
            loop {
                let packet = reader.read().await;
                if packet.id == clientbound::JoinGame::ID {
                    if let Some(os_tx) = oneshot.take() {
                        let mut packet = packet.clone();
                        let processed: JoinGame = packet.read();
                        os_tx.send(processed.entity_id).unwrap();
                    }
                }
                match tx.send(packet) {
                    Ok(ok) => {}
                    Err(err) => {
                        // the other end is stopped and should have printed the error
                        return;
                    }
                }
            }
        });

        let tx = writer.into_channel();

        let entity_id = match os_rx.await {
            Ok(inner) => inner,
            Err(err) => {
                return Err(crate::error::err("disconnected before join game packet"));
            }
        };

        let protocol = Protocol {
            rx,
            tx,
            disconnected: false,
        };

        let login = Login {
            protocol,
            info: ClientInfo {
                username,
                uuid,
                entity_id,
            },
        };

        Ok(login)
    }

    fn apply_packets(&mut self, client: &mut State, global: &mut GlobalState) {
        loop {
            match self.rx.try_recv() {
                Ok(data) => {
                    self.process_packet(data, client);
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            println!("disconnected because error");
                            self.disconnected = true;
                        }
                    }
                    return;
                }
            }
        }
    }

    fn send_chat(&mut self, message: &str) {
        self.tx.write(serverbound::Chat::message(message));
    }

    fn teleport(&mut self) {}

    fn disconnected(&self) -> bool {
        self.disconnected
    }
}

impl Protocol {
    fn process_packet(&mut self, mut data: PacketData, client: &mut State) {
        use clientbound::*;
        match data.id {
            JoinGame::ID => println!("{} joined", client.info.username),
            KeepAlive::ID => {
                let KeepAlive { id } = data.read();

                self.tx.write(serverbound::KeepAlive {
                    id
                });
            }
            UpdateHealth::ID => {
                let UpdateHealth { health, .. } = data.read();
                if health <= 0.0 {
                    if client.alive {
                        client.alive = false;
                        self.tx.write(serverbound::ClientStatus {
                            action: serverbound::ClientStatusAction::Respawn
                        });
                        self.tx.write(serverbound::Chat::message("I'm respawning"));
                    }
                } else {
                    client.alive = true;
                }
            }
            PlayerPositionAndLook::ID => {
                let PlayerPositionAndLook { location, rotation, teleport_id } = data.read();
                self.tx.write(serverbound::TeleportConfirm {
                    teleport_id
                });
            }
            PlayDisconnect::ID => {
                let PlayDisconnect { reason } = data.read();
                println!("player disconnected ... {}", reason);
                self.disconnected = true;
            }
            ChatMessage::ID => {

                let rdm = thread_rng().gen_range(0..100);
                if rdm == 0 {
                    let ChatMessage { json, .. } = data.read();
                    println!("chat {}", json);
                }
            }
            _ => {}
        }
    }
}
