use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{SendError, TryRecvError};

use packets::types::{PacketState, UUID, VarInt};
use packets::types::Packet;
use packets::write::ByteWritable;
use rand::{Rng, thread_rng};
use tokio::sync::oneshot::error::RecvError;
use tokio::time::{Duration, sleep};

use crate::bootstrap::{Address, Connection, CSVUser};
use crate::bootstrap::mojang::{AuthResponse, calc_hash, Mojang};
use crate::bootstrap::storage::ValidUser;
use crate::client::processor::InterfaceIn;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::error::Error::WrongPacket;
use crate::error::Res;
use crate::protocol::{ClientInfo, EventQueue, InterfaceOut, Login, Minecraft};
use crate::protocol::encrypt::{rand_bits, RSA};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::{PacketWriteChannel, PacketWriter};
use crate::protocol::v340::clientbound::{Disconnect, JoinGame, LoginSuccess};
use crate::protocol::v340::serverbound::{ClientStatusAction, HandshakeNextState, TeleportConfirm};
use crate::storage::world::ChunkLocation;
use crate::types::{Location, PacketData};

mod clientbound;
mod serverbound;

pub struct EventQueue340 {
    rx: std::sync::mpsc::Receiver<PacketData>,
    out: Interface340,
    disconnected: bool,
    location: Location,

    /// we need to store state because sometimes death packets occur twice and we only want to send one event
    alive: bool,
}

impl EventQueue for EventQueue340 {
    fn flush(&mut self, processor: &mut impl InterfaceIn) {
        loop {
            match self.rx.try_recv() {
                Ok(data) => {
                    self.process_packet(data, processor);
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
}

impl EventQueue340 {
    fn process_packet(&mut self, mut data: PacketData, processor: &mut impl InterfaceIn) {
        use clientbound::*;
        match data.id {
            JoinGame::ID => {}
            KeepAlive::ID => {
                // auto keep alive
                let KeepAlive { id } = data.read();

                self.out.write(serverbound::KeepAlive {
                    id
                });
            }
            UpdateHealth::ID => {
                let UpdateHealth { health, .. } = data.read();
                if health <= 0.0 && self.alive {
                    processor.on_death();
                    self.alive = false;
                }
            }
            ChunkColumnPacket::ID => {
                let ChunkColumnPacket { chunk_x, chunk_z, column } = data.read();
                processor.on_recv_chunk(ChunkLocation(chunk_x, chunk_z), column);
            }
            PlayerPositionAndLook::ID => {
                let PlayerPositionAndLook { location, rotation, teleport_id } = data.read();

                self.location.apply_change(location);
                processor.on_move(self.location);

                // "accept" the packet
                self.out.write(serverbound::TeleportConfirm {
                    teleport_id
                });
            }
            PlayDisconnect::ID => {
                let PlayDisconnect { reason } = data.read();
                processor.on_disconnect(&reason);
                self.disconnected = true;
            }
            // ignore
            ChatMessage::ID => {
                let ChatMessage { chat, position } = data.read();
                processor.on_chat(chat);
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct Interface340 {
    tx: Rc<RefCell<PacketWriteChannel>>,
}

impl Interface340 {
    fn new(tx: PacketWriteChannel) -> Interface340 {
        Interface340 {
            tx: Rc::new(RefCell::new(tx))
        }
    }

    #[inline]
    fn write<T: Packet + ByteWritable>(&self, packet: T) {
        self.tx.borrow_mut().write(packet)
    }
}

impl InterfaceOut for Interface340 {
    fn send_chat(&mut self, message: &str) {
        self.write(serverbound::ChatMessage {
            message: message.to_string()
        });
    }

    fn respawn(&mut self) {
        self.write(serverbound::ClientStatus {
            action: ClientStatusAction::Respawn
        });
    }

    fn teleport(&mut self, location: Location) {
        self.write(serverbound::PlayerPosition {
            location,
            on_ground: true,
        });
    }
}

pub struct Protocol;

#[async_trait::async_trait]
impl Minecraft for Protocol {
    type Queue = EventQueue340;
    type Interface = Interface340;

    async fn login(conn: Connection) -> Res<Login<EventQueue340, Interface340>> {
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

        let out = Interface340::new(tx);

        let queue = EventQueue340 {
            rx,
            out: out.clone(),
            disconnected: false,
            location: Default::default(),
            alive: false,
        };

        let login = Login {
            queue,
            out,
            info: ClientInfo {
                username,
                uuid,
                entity_id,
            },
        };

        Ok(login)
    }
}
