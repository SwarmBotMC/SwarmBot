use std::sync::mpsc::{SendError, TryRecvError};

use packets::types::{PacketState, UUID, VarInt};
use packets::types::Packet;
use tokio::sync::oneshot::error::RecvError;
use tokio::time::{Duration, sleep};

use crate::bootstrap::{Connection, CSVUser, Address};
use crate::bootstrap::mojang::{AuthResponse, calc_hash, Mojang};
use crate::client::instance::{ClientInfo};
use crate::error::Error::WrongPacket;
use crate::error::Res;
use crate::protocol::{Login, McProtocol};
use crate::protocol::encrypt::{rand_bits, RSA};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::{PacketWriteChannel, PacketWriter};
use crate::types::{PacketData, Location};
use crate::protocol::v340::clientbound::{Disconnect, JoinGame, LoginSuccess};
use crate::protocol::v340::serverbound::{HandshakeNextState, TeleportConfirm};
use rand::{thread_rng, Rng};
use crate::storage::world::ChunkLocation;
use crate::bootstrap::storage::ValidUser;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;

mod clientbound;
mod serverbound;

pub struct Protocol {
    rx: std::sync::mpsc::Receiver<PacketData>,
    tx: PacketWriteChannel,
    disconnected: bool,
}

#[async_trait::async_trait]
impl McProtocol for Protocol {

    async fn login(conn: Connection) -> Res<Login<Self>> {
        let Connection { user, address, mojang, read, write } = conn;
        let ValidUser { email, username, password, last_checked, uuid, access_id, client_id } = user;

        let Address { host, port } = address;
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
        mojang.join(uuid, &hash, &access_id).await?;

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

    fn apply_packets(&mut self, client: &mut LocalState, global: &mut GlobalState) {
        loop {
            match self.rx.try_recv() {
                Ok(data) => {
                    self.process_packet(data, client, global);
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
        self.tx.write(serverbound::ChatMessage::message(message));
    }

    fn teleport(&mut self, location: Location) {
        self.tx.write(serverbound::PlayerPosition {
            location,
            on_ground: true
        })
    }

    fn disconnected(&self) -> bool {
        self.disconnected
    }
}

impl Protocol {
    fn process_packet(&mut self, mut data: PacketData, client: &mut LocalState, global: &mut GlobalState) {
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
                        self.tx.write(serverbound::ChatMessage::message("I'm respawning"));
                    }
                } else {
                    client.alive = true;
                }
            }
            ChunkColumnPacket::ID => {
                let ChunkColumnPacket{ chunk_x, chunk_z, column } = data.read();
                global.world_blocks.add_column(ChunkLocation(chunk_x, chunk_z), column)
            }
            PlayerPositionAndLook::ID => {
                let PlayerPositionAndLook { location, rotation, teleport_id } = data.read();

                // update the client's location
                client.location.apply_change(location);

                // "accept" the packet
                self.tx.write(serverbound::TeleportConfirm {
                    teleport_id
                });
            }
            PlayDisconnect::ID => {
                let PlayDisconnect { reason } = data.read();
                println!("player disconnected ... {}", reason);
                self.disconnected = true;
            }
            // ignore
            ChatMessage::ID => {
                let ChatMessage{ chat, position } = data.read();
                client.process_chat(chat);
            }
            _ => {}
        }
    }
}
