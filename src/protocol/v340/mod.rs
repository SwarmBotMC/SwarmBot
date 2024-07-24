use std::{cell::RefCell, rc::Rc, sync::mpsc::TryRecvError};

use anyhow::{bail, Context};
use interfaces::types::{BlockLocation, BlockState, ChunkLocation};
use swarm_bot_packets::{
    types::{Packet, VarInt, UUID},
    write::ByteWritable,
};

use crate::{
    bootstrap::{mojang::calc_hash, storage::BotData, Address, BotConnection},
    client::processor::InterfaceIn,
    protocol::{
        encrypt::{rand_bits, Rsa},
        io::{
            reader::PacketReader,
            writer::{PacketWriteChannel, PacketWriter},
        },
        v340::{
            clientbound::{EncryptionRequest, JoinGame, LoginDisconnect, LoginSuccess},
            serverbound::{
                ClientStatusAction, DigStatus, Hand, HandshakeNextState, InteractEntityKind,
            },
        },
        ClientInfo, EventQueue, Face, InterfaceOut, InvAction, Login, Mine, Minecraft,
    },
    storage::entities::EntityKind,
    types::{Dimension, Direction, Location, PacketData, Slot},
};

mod clientbound;
mod serverbound;

pub struct EventQueue340 {
    rx: std::sync::mpsc::Receiver<PacketData>,
    out: Interface340,
    location: Location,
    dimension: Dimension,

    /// we need to store state because sometimes death packets occur twice and
    /// we only want to send one event
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
    // allow lines
    #[allow(clippy::too_many_lines)]
    fn process_packet(&mut self, mut data: PacketData, processor: &mut impl InterfaceIn) {
        use clientbound::*;
        match data.id {
            JoinGame::ID => {
                let JoinGame { dimension, .. } = data.read();
                processor.on_join();
                processor.on_dimension_change(dimension);
            }

            window::Set::ID => {
                let window::Set {
                    window_id,
                    slot: idx,
                    data,
                } = data.read();
                if window_id == 0 {
                    match data.into() {
                        None => processor.on_lose_item(idx as usize),
                        Some(item_stack) => processor.on_pickup_item(idx as usize, item_stack),
                    }
                }
            }

            window::Items::ID => {
                let window::Items { window_id, slots } = data.read();

                if window_id == 0 {
                    // is player inventory
                    for (idx, slot) in slots.0.into_iter().enumerate() {
                        match slot.into() {
                            None => processor.on_lose_item(idx),
                            Some(item_stack) => processor.on_pickup_item(idx, item_stack),
                        }
                    }
                }
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

                self.out.write(serverbound::KeepAlive { id });
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
                let entity::Teleport {
                    entity_id,
                    location,
                    ..
                } = data.read();
                processor.on_entity_move(entity_id.into(), location.into());
            }
            entity::LivingSpawn::ID => {
                let entity::LivingSpawn {
                    entity_id,
                    location,
                    ..
                } = data.read();
                processor.on_entity_spawn(entity_id.into(), location, EntityKind::Normal);
            }
            entity::PlayerSpawn::ID => {
                let entity::PlayerSpawn {
                    entity_id,
                    location,
                    player_uuid,
                    ..
                } = data.read();
                processor.on_entity_spawn(
                    entity_id.into(),
                    location,
                    EntityKind::Player {
                        uuid: player_uuid.0,
                    },
                );
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
            CHUNK_PKT_ID => {
                let overworld = self.dimension == Dimension::Overworld;
                let ChunkColumnPacket {
                    chunk_x,
                    chunk_z,
                    column,
                    new_chunk,
                } = data.reader.read_like(&overworld);
                processor.on_recv_chunk(ChunkLocation(chunk_x, chunk_z), column, new_chunk);
            }
            MultiBlock::ID => {
                let MultiBlock {
                    chunk_x,
                    chunk_z,
                    records,
                } = data.read();

                let base_x = chunk_x << 4;
                let base_z = chunk_z << 4;

                for Record {
                    x,
                    y,
                    z,
                    block_state,
                } in records
                {
                    let location = BlockLocation::new(
                        base_x + i32::from(x),
                        i16::from(y),
                        base_z + i32::from(z),
                    );
                    processor.on_block_change(location, BlockState(block_state.0 as u32));
                }
            }
            PlayerPositionAndLook::ID => {
                let PlayerPositionAndLook {
                    location,
                    teleport_id,
                    ..
                } = data.read();

                self.location.apply_change(location);
                processor.on_move(self.location);

                // "accept" the packet
                self.out.write(serverbound::TeleportConfirm { teleport_id });
            }
            PlayDisconnect::ID => {
                let PlayDisconnect { reason } = data.read();
                processor.on_disconnect(&reason);
            }
            PlayerListItem::ID => {
                let PlayerListItem { players } = data.read();
                for Player { uuid, list_type } in players {
                    match list_type {
                        PlayerListType::AddPlayer(add) => {
                            processor.on_player_join(uuid.0, add.name);
                        }
                        PlayerListType::RemovePlayer => processor.on_player_leave(uuid.0),
                        _ => {}
                    }
                }
            }
            // ignore
            ChatMessage::ID => {
                let ChatMessage { chat, .. } = data.read();
                processor.on_chat(chat);
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct Interface340 {
    tx: Rc<RefCell<PacketWriteChannel>>,
    inv_action_id: u16,
}

impl Interface340 {
    fn new(tx: PacketWriteChannel) -> Self {
        Self {
            tx: Rc::new(RefCell::new(tx)),
            inv_action_id: 0,
        }
    }

    fn click(&mut self, slot: u16, button: impl Into<u8>, mode: i32, clicked: impl Into<Slot>) {
        let action_number = self.inv_action_id;
        let to_send = serverbound::ClickWindow {
            window_id: 0,
            slot,
            button: button.into(),
            action_number,
            mode: VarInt(mode),
            clicked: clicked.into(),
        };

        self.write(to_send);

        self.inv_action_id += 1;
    }

    #[inline]
    fn write<T: Packet + ByteWritable>(&self, packet: T) {
        self.tx.borrow_mut().write(packet);
    }
}

impl InterfaceOut for Interface340 {
    fn place_block(&mut self, against: BlockLocation, face: Face) {
        let face = VarInt(face as i32);

        self.write(serverbound::PlaceBlock {
            location: against,
            face,
            hand: Hand::Main,
            cursor: serverbound::BlockCursor {
                x: 1.0,
                y: 1.5,
                z: 0.5,
            },
        });
    }

    fn attack_entity(&mut self, id: u32) {
        self.write(serverbound::InteractEntity {
            id: id.into(),
            kind: InteractEntityKind::Attack,
        });
    }

    fn send_chat(&mut self, message: &str) {
        self.write(serverbound::ChatMessage {
            message: message.to_string(),
        });
    }

    fn inventory_action(&mut self, action: InvAction) {
        match action {
            InvAction::Q(slot) => self.click(slot, 0, 4, Slot::EMPTY),
            InvAction::CtrlQ(slot) => self.click(slot, 1, 4, Slot::EMPTY),
            InvAction::Click(slot, button, clicked) => self.click(slot, button, 0, clicked),
            InvAction::ShiftClick(slot, button, clicked) => self.click(slot, button, 1, clicked),
        }
    }

    fn swing_arm(&mut self) {
        self.write(serverbound::ArmAnimation { hand: Hand::Main });
    }

    fn finish_eating(&mut self) {
        self.write(serverbound::PlayerDig::status(
            DigStatus::ShootArrowOrFinishEat,
        ));
    }

    fn use_item(&mut self) {
        self.write(serverbound::UseItem { hand: Hand::Main });
    }

    fn change_slot(&mut self, number: u8) {
        self.write(serverbound::ChangeSlot {
            slot: u16::from(number),
        });
    }

    fn mine(&mut self, position: BlockLocation, mine: Mine, face: Face) {
        let status = match mine {
            Mine::Start => DigStatus::Started,
            Mine::Cancel => DigStatus::Cancelled,
            Mine::Finished => DigStatus::Finished,
        };

        if status == DigStatus::Started {
            self.swing_arm();
        }

        self.write(serverbound::PlayerDig {
            status,
            position,
            face: face as u8,
        });
    }

    fn respawn(&mut self) {
        self.write(serverbound::ClientStatus {
            action: ClientStatusAction::Respawn,
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
        });
    }

    fn teleport_and_look(&mut self, location: Location, direction: Direction, on_ground: bool) {
        self.write(serverbound::PlayerPositionAndRotation {
            location,
            direction,
            on_ground,
        });
    }
}

pub struct Protocol;

impl Minecraft for Protocol {
    type Queue = EventQueue340;
    type Interface = Interface340;

    /// Login for 1.12.2
    ///
    /// Look <https://wiki.vg/index.php?title=Protocol&oldid=14204#Login>
    async fn login(conn: BotConnection) -> anyhow::Result<Login<EventQueue340, Interface340>> {
        let BotConnection {
            server_address,
            bot,
            read,
            write,
        } = conn;

        let username = bot.username();

        let Address { host, port } = server_address;

        let mut reader = PacketReader::from(read);
        let mut writer = PacketWriter::from(write);

        // ----------- START: handshake ------------
        // 1. C -> S
        writer
            .write(serverbound::Handshake {
                protocol_version: VarInt(340),
                host,
                port,
                next_state: HandshakeNextState::Login,
            })
            .await?;

        // ------------ START: login ----------------
        // 2 C -> S     LOGIN START
        writer
            .write(serverbound::LoginStart {
                username: username.to_string(),
            })
            .await?;

        // 3. S -> C: Encryption Request

        let uuid = if let BotData::Online { user, mojang } = &bot {
            let access_id = &user.access_id;
            let uuid = user.uuid();

            let clientbound::EncryptionRequest {
                public_key_der,
                verify_token,
                server_id,
            } = reader.read_exact_packet().await?;

            let rsa = Rsa::from_der(&public_key_der);

            let shared_secret = rand_bits();

            let encrypted_ss = rsa.encrypt(&shared_secret).unwrap();
            let encrypted_verify = rsa.encrypt(&verify_token).unwrap();

            // 4. Mojang online mode requests
            let hash = calc_hash(&server_id, &shared_secret, &public_key_der);
            mojang.join(uuid, &hash, access_id).await?;

            // 5. Encryption Response
            writer
                .write(serverbound::EncryptionResponse {
                    shared_secret: encrypted_ss,
                    verify_token: encrypted_verify,
                })
                .await?;

            // we now do everything encrypted
            writer.encryption(&shared_secret);
            reader.encryption(&shared_secret);

            uuid
        } else {
            // TODO: remove
            UUID::default()
        };

        // set compression or login success
        let mut data = reader.read().await?;

        let LoginSuccess { .. } = match data.id {
            clientbound::SetCompression::ID => {
                let clientbound::SetCompression { threshold } = data.read();

                reader.compression(threshold.into());
                writer.compression(threshold.into());

                reader.read_exact_packet().await?
            }
            LoginSuccess::ID => data.reader.read(),
            LoginDisconnect::ID => {
                let LoginDisconnect { reason } = data.reader.read();
                let reason = reason.colorize();
                bail!("Disconnected while logging in. Reason: {reason}")
            }
            EncryptionRequest::ID => {
                bail!("Server requested encryption but, we are in offline mode")
            }
            actual => {
                let expected = LoginSuccess::ID;
                bail!(
                    "wrong packet ID for logging in. Expected \
                ID {expected} which is the ID for login success but, got ID of {actual}."
                )
            }
        };

        let (tx, rx) = std::sync::mpsc::channel();
        let (os_tx, os_rx) = tokio::sync::oneshot::channel();

        tokio::task::spawn_local(async move {
            let mut oneshot = Some(os_tx);
            loop {
                let packet = reader.read().await.unwrap();
                if packet.id == JoinGame::ID {
                    if let Some(os_tx) = oneshot.take() {
                        let mut packet = packet.clone();
                        let processed: JoinGame = packet.read();
                        os_tx
                            .send((processed.entity_id, processed.dimension))
                            .unwrap();
                    }
                }
                match tx.send(packet) {
                    Ok(..) => {}
                    Err(..) => {
                        // the other end is stopped and should have printed the error
                        return;
                    }
                }
            }
        });

        let tx = writer.into_channel();

        let (entity_id, dimension) = os_rx
            .await
            .context("disconnected before join game packet")?;

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
                username: username.to_string(),
                uuid,
                entity_id,
            },
        };

        Ok(login)
    }
}
