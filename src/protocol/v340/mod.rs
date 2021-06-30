/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;

use packets::types::{PacketState, UUID, VarInt};
use packets::types::Packet;
use packets::write::ByteWritable;

use crate::bootstrap::{Address, Connection};
use crate::bootstrap::mojang::calc_hash;
use crate::bootstrap::storage::ValidUser;
use crate::client::processor::InterfaceIn;
use crate::error::Error::WrongPacket;
use crate::error::Res;
use crate::protocol::{ClientInfo, EventQueue, InterfaceOut, Login, Mine, Minecraft};
use crate::protocol::encrypt::{rand_bits, RSA};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::{PacketWriteChannel, PacketWriter};
use crate::protocol::v340::clientbound::{JoinGame, LoginSuccess};
use crate::protocol::v340::serverbound::{ClientStatusAction, DigStatus, Hand, HandshakeNextState};
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::blocks::ChunkLocation;
use crate::types::{Dimension, Direction, Location, PacketData};

mod clientbound;
mod serverbound;

pub struct EventQueue340 {
    rx: std::sync::mpsc::Receiver<PacketData>,
    out: Interface340,
    location: Location,
    dimension: Dimension,

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
                            processor.on_socket_close();
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
            JoinGame::ID => {
                let JoinGame { dimension, .. } = data.read();
                processor.on_dimension_change(dimension);
            }

            BlockChange::ID => {
                let BlockChange { block_id, location } = data.read();
                processor.on_block_change(location, BlockState(block_id.0 as u32));
            }
            Explosion::ID => {
                let Explosion { records, .. } = data.read();

                for record in records {
                    processor.on_block_change(record, BlockState::AIR);
                }
            }
            KeepAlive::ID => {
                // auto keep alive
                let KeepAlive { id } = data.read();

                self.out.write(serverbound::KeepAlive {
                    id
                });
            }
            entity::RelativeMove::ID => {
                let entity::RelativeMove { entity_id, loc, .. } = data.read();
                processor.on_entity_move(entity_id.into(), loc.into());
            }
            entity::LookAndRelativeMove::ID => {
                let entity::LookAndRelativeMove { entity_id, loc, .. } = data.read();
                processor.on_entity_move(entity_id.into(), loc.into());
            }
            entity::Destroy::ID => {
                let entity::Destroy { ids } = data.read();
                for id in ids {
                    processor.on_entity_destroy(id.into());
                }
            }
            entity::Teleport::ID => {
                let entity::Teleport { entity_id, location, .. } = data.read();
                processor.on_entity_move(entity_id.into(), location.into());
            }
            entity::LivingSpawn::ID => {
                let entity::LivingSpawn { entity_id, location, .. } = data.read();
                processor.on_entity_spawn(entity_id.into(), location);
            }
            entity::PlayerSpawn::ID => {
                let entity::PlayerSpawn { entity_id, location, .. } = data.read();
                processor.on_entity_spawn(entity_id.into(), location);
            }
            UpdateHealth::ID => {
                let UpdateHealth { health, food, .. } = data.read();
                if health > 0.0 {
                    processor.on_update_health(health, food.0 as u8);
                    self.alive = true;
                } else if self.alive {
                    processor.on_death();
                    self.alive = false;
                }
            }
            Respawn::ID => {
                let Respawn { dimension, .. } = data.read();
                processor.on_dimension_change(dimension);
                self.dimension = dimension;
            }

            // need to do this because the chunk packet is read differently based on dimension
            clientbound::CHUNK_PKT_ID => {
                let overworld = self.dimension == Dimension::Overworld;
                let ChunkColumnPacket { chunk_x, chunk_z, column, new_chunk } = data.reader.read_like(&overworld);
                processor.on_recv_chunk(ChunkLocation(chunk_x, chunk_z), column, new_chunk);
            }
            MultiBlock::ID => {
                let MultiBlock { chunk_x, chunk_z, records } = data.read();

                let base_x = chunk_x << 4;
                let base_z = chunk_z << 4;

                for Record { x, y, z, block_state } in records {
                    let location = BlockLocation::new(base_x + x as i32, y as i16, base_z + z as i32);
                    processor.on_block_change(location, BlockState(block_state.0 as u32))
                }
            }
            PlayerPositionAndLook::ID => {
                let PlayerPositionAndLook { location, rotation: _, teleport_id } = data.read();

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
            }
            // ignore
            ChatMessage::ID => {
                let ChatMessage { chat, position: _ } = data.read();
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

    fn left_click(&mut self) {
        self.write(serverbound::ArmAnimation {
            hand: Hand::Main
        });
    }

    fn mine(&mut self, position: BlockLocation, mine: Mine) {
        let status = match mine {
            Mine::Start => DigStatus::Started,
            Mine::Cancel => DigStatus::Cancelled,
            Mine::Finished => DigStatus::Finished
        };

        if status == DigStatus::Started {
            self.left_click();
        }

        self.write(serverbound::PlayerDig {
            status,
            position,
            face: 1,
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

    fn look(&mut self, direction: Direction) {
        self.write(serverbound::PlayerLook {
            direction,
            on_ground: false,
        })
    }

    fn teleport_and_look(&mut self, location: Location, direction: Direction, on_ground: bool) {
        self.write(serverbound::PlayerPositionAndRotation {
            location,
            direction,
            on_ground,
        })
    }
}

pub struct Protocol;

#[async_trait::async_trait]
impl Minecraft for Protocol {
    type Queue = EventQueue340;
    type Interface = Interface340;

    async fn login(conn: Connection) -> Res<Login<EventQueue340, Interface340>> {
        let Connection { user, address, mojang, read, write } = conn;
        let ValidUser { email: _, username, password: _, last_checked: _, uuid, access_id, client_id: _ } = user;

        let Address { host, port } = address;
        let uuid = UUID::from(&uuid);

        let mut reader = PacketReader::from(read);
        let mut writer = PacketWriter::from(write);

        let _online = true;

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

            let LoginSuccess { username: _, uuid: _ } = match data.id {
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
                        os_tx.send((processed.entity_id, processed.dimension)).unwrap();
                    }
                }
                match tx.send(packet) {
                    Ok(_ok) => {}
                    Err(_err) => {
                        // the other end is stopped and should have printed the error
                        return;
                    }
                }
            }
        });

        let tx = writer.into_channel();

        let (entity_id, dimension) = match os_rx.await {
            Ok(inner) => inner,
            Err(_err) => {
                return Err(crate::error::err("disconnected before join game packet"));
            }
        };

        let out = Interface340::new(tx);

        let queue = EventQueue340 {
            rx,
            dimension,
            out: out.clone(),
            location: Default::default(),
            alive: true,
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
