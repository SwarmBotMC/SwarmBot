/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::cmp::max;

use itertools::Itertools;
use packets::{Packet, Readable, Writable};
use packets::read::{ByteReadable, ByteReadableLike, ByteReader};
use packets::types::{BitField, Identifier, RawVec, UUIDHyphenated, VarInt, VarUInt};

use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::chunk::{ChunkColumn, ChunkData, HighMemoryChunkSection, Palette};
use crate::types::{Chat, Dimension, Direction, DirectionOrigin, Location, LocationFloat, LocationOrigin, Position};

#[derive(Packet, Readable)]
#[packet(0x00, Login)]
pub struct Disconnect {
    pub reason: String,
}


#[derive(Debug)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl ByteReadable for GameMode {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        use GameMode::*;
        let val: u8 = byte_reader.read();
        match val {
            0 => Survival,
            1 => Creative,
            2 => Adventure,
            3 => Spectator,
            val => panic!("gamemode {} is not valid", val)
        }
    }
}

#[derive(Debug)]
pub struct Record {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub block_state: VarUInt,
}

impl ByteReadable for Record {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let horizontal: u8 = byte_reader.read();
        let y: u8 = byte_reader.read();
        Self {
            x: horizontal >> 4,
            y,
            z: horizontal << 4,
            block_state: byte_reader.read(),
        }
    }
}

#[derive(Packet, Debug, Readable)]
#[packet(0x10, Play)]
pub struct MultiBlock {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub records: Vec<Record>,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x23, Play)]
pub struct JoinGame {
    pub entity_id: u32,
    pub game_mode: GameMode,
    pub dimension: Dimension,
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    pub reduced_debug_info: bool,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x35, Play)]
pub struct Respawn {
    pub dimension: Dimension,
    pub difficulty: u8,
    pub gamemode: GameMode,
    pub level_type: String,
}

#[derive(Packet, Readable)]
#[packet(0x03, Login)]
pub struct SetCompression {
    pub threshold: VarInt,
}

#[derive(Debug, Clone, Packet, Writable, Readable)]
#[packet(0x01, Login)]
pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key_der: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Debug, Packet, Readable)]
#[packet(0x02, Login)]
pub struct LoginSuccess {
    pub uuid: UUIDHyphenated,
    // 1.16 this is just a raw UUID, 1.12 hyphenated
    pub username: String,
}

#[derive(Readable)]
pub struct PlayerPositionAndLookRaw {
    location: Location,
    rotation: Direction,
    flags: BitField,
    teleport_id: VarInt,
}

pub mod entity {
    use packets::*;
    use packets::types::{Angle, UUID, VarInt};

    use crate::types::{Location, ShortLoc};

    #[derive(Packet, Debug, Readable)]
    #[packet(0x03, Play)]
    pub struct LivingSpawn {
        pub entity_id: VarInt,
        pub entity_uuid: UUID,
        pub type_id: VarInt,
        pub location: Location,
        pub yaw: Angle,
        pub pitch: Angle,
        pub head_pitch: Angle,
        pub velocity_x: i16,
        pub velocity_y: i16,
        pub velocity_z: i16,
    }

    #[derive(Packet, Debug, Readable)]
    #[packet(0x05, Play)]
    pub struct PlayerSpawn {
        pub entity_id: VarInt,
        pub player_uuid: UUID,
        pub location: Location,
        pub yaw: Angle,
        pub pitch: Angle,
        // TODO metadata
        // pub head_pitch: Angle,
        // pub velocity_x: i16,
        // pub velocity_y: i16,
        // pub velocity_z: i16,
    }

    #[derive(Packet, Debug, Readable)]
    #[packet(0x26, Play)]
    pub struct RelativeMove {
        pub entity_id: VarInt,
        pub loc: ShortLoc,
        pub on_ground: bool,
    }

    #[derive(Packet, Debug, Readable)]
    #[packet(0x32, Play)]
    pub struct Destroy {
        pub ids: Vec<VarInt>,
    }

    #[derive(Packet, Debug, Readable)]
    #[packet(0x27, Play)]
    pub struct LookAndRelativeMove {
        pub entity_id: VarInt,
        pub loc: ShortLoc,
        pub yaw: Angle,
        pub pitch: Angle,
        pub on_ground: bool,
    }

    #[derive(Packet, Debug, Readable)]
    #[packet(0x4c, Play)]
    pub struct Teleport {
        pub entity_id: VarInt,
        pub location: Location,
        pub yaw: Angle,
        pub pitch: Angle,
        pub on_ground: bool,
    }
}


#[derive(Packet, Debug)]
#[packet(0x2f, Play)]
pub struct PlayerPositionAndLook {
    pub location: LocationOrigin,
    pub rotation: DirectionOrigin,
    pub teleport_id: VarInt,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x17, Play)]
pub struct PluginMessage {
    pub channel: Identifier,
    pub data: RawVec,
}


#[derive(Packet, Debug, Readable)]
#[packet(0x0f, Play)]
pub struct ChatMessage {
    pub chat: Chat,
    pub position: u8,
}

#[derive(Readable, Debug)]
pub struct Offset {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

#[derive(Packet, Debug)]
#[packet(0x1c, Play)]
pub struct Explosion {
    pub location: Location,
    pub radius: f32,
    pub records: Vec<BlockLocation>,
    // TODO: velocity
}

impl ByteReadable for Explosion {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let location: LocationFloat = byte_reader.read();
        let radius = byte_reader.read();
        let records = {
            let record_size: i32 = byte_reader.read();
            let record_size = record_size as usize;
            let records: RawVec<Offset> = byte_reader.read_like(&record_size);
            records.0
        };


        let origin_block = BlockLocation::from_flts(location.x, location.y, location.z);
        let location: Location = location.into();
        let records = records.into_iter().map(|record| BlockLocation::new(
            origin_block.x + record.x as i32,
            origin_block.y + record.y as i16,
            origin_block.z + record.z as i32,
        )).collect();
        Self {
            location,
            radius,
            records,
        }
    }
}

#[derive(Packet, Debug, Readable)]
#[packet(0x0b, Play)]
pub struct BlockChange {
    pub location: Position,
    pub block_id: VarInt,
}


#[derive(Packet, Debug, Readable)]
#[packet(0x41, Play)]
pub struct UpdateHealth {
    pub health: f32,

    // 0-20
    pub food: VarInt,
    pub food_saturation: f32,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x1f, Play)]
pub struct KeepAlive {
    pub id: u64,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x1a, Play)]
pub struct PlayDisconnect {
    pub reason: String,
}

// #[derive(Packet)]
// #[packet(0x20, Play)]

pub const CHUNK_PKT_ID: u32 = 0x20;

pub struct ChunkColumnPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub new_chunk: bool,
    pub column: ChunkColumn,
}

impl ByteReadableLike for ChunkColumnPacket {
    type Param = bool;

    fn read_from_bytes(byte_reader: &mut ByteReader, param: &Self::Param) -> Self {
        let chunk_x = byte_reader.read();
        let chunk_z = byte_reader.read();
        let ground_up_continuous: bool = byte_reader.read();
        let VarUInt(mut primary_bitmask) = byte_reader.read();
        let _size: VarUInt = byte_reader.read();

        const INIT: Option<HighMemoryChunkSection> = None;
        let mut sections = [INIT; 16];

        let mut idx = 0;
        while primary_bitmask != 0 {
            if primary_bitmask & 0b1 == 1 {
                let section: ChunkSection = byte_reader.read_like(param);
                sections[idx] = Some(HighMemoryChunkSection::new(section.palette));
            }
            primary_bitmask >>= 1;
            idx += 1;
        }

        let data = ChunkData {
            sections
        };

        let column = ChunkColumn::HighMemory {
            data
        };

        ChunkColumnPacket {
            chunk_x,
            chunk_z,
            new_chunk: ground_up_continuous,
            column,
        }
    }
}


pub mod window {
    use crate::types::{ShortVec, Slot};

    #[derive(Packet, Debug, Readable)]
    #[packet(0x16, Play)]
    pub struct Set {
        pub window_id: u8,
        pub slot: u16,
        pub data: Slot
    }

    #[derive(Packet, Debug, Readable)]
    #[packet(0x13, Play)]
    pub struct Open {

        pub window_id: u8,

        /// https://wiki.vg/Inventory
        pub kind: String,

        pub title: String,

        pub slot_count: u8,

        // TODO: entity id if EntityHorse

    }

    #[derive(Packet, Readable)]
    #[packet(0x14, Play)]
    pub struct Items {
        /// 0 if player inventory
        pub window_id: u8,

        // number of elements in the array
        pub slots: ShortVec<Slot>
    }

}


pub struct ChunkSection {
    palette: crate::storage::chunk::Palette,

    #[allow(unused)]
    block_light: [u8; 2048],

    // TODO: fix overworld
    #[allow(unused)]
    sky_light: Option<[u8; 2048]>,
}


impl ByteReadableLike for ChunkSection {
    type Param = bool;

    fn read_from_bytes(byte_reader: &mut ByteReader, param: &Self::Param) -> Self {
        let bits_per_block: u8 = byte_reader.read();
        let palette = if bits_per_block <= 8 {
            let bits_per_block = max(bits_per_block, 4);
            let block_state_ids: Vec<VarInt> = byte_reader.read();
            let block_state_ids = block_state_ids.into_iter().map(|id| BlockState(id.0 as u32)).collect_vec();
            let storage: Vec<u64> = byte_reader.read();
            Palette::indirect(bits_per_block, block_state_ids, storage)
        } else {
            let VarInt(_place_holder) = byte_reader.read();
            let storage: Vec<u64> = byte_reader.read();
            Palette::direct(storage)
        };

        let block_light = byte_reader.read();
        let sky_light = param.then(|| byte_reader.read());
        ChunkSection {
            palette,
            block_light,
            sky_light,
        }
    }
    // fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
    // }
}

impl ByteReadable for PlayerPositionAndLook {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let raw: PlayerPositionAndLookRaw = byte_reader.read();
        raw.into_origin()
    }
}

impl PlayerPositionAndLookRaw {
    fn into_origin(self) -> PlayerPositionAndLook {
        let [x, y, z, y_rot, x_rot, ..] = self.flags.values;

        let location = LocationOrigin::from(self.location, x, y, z);
        let rotation = DirectionOrigin::from(self.rotation, y_rot, x_rot);

        PlayerPositionAndLook {
            location,
            rotation,
            teleport_id: self.teleport_id,
        }
    }
}
