use packets::types::{VarInt, UUIDHyphenated, BitField, UUID, Angle, Identifier, RawVec, Position, VarUInt};
use packets::{Readable, Writable, Packet, EnumReadable};
use packets::read::{ByteReadable, ByteReader};
use crate::types::{LocationOrigin, DirectionOrigin, Location, ShortLoc, Direction};
use std::cmp::max;
use crate::storage::chunk::{Palette, ChunkColumn, ChunkData, LowMemoryChunkSection, HighMemoryChunkSection};
use crate::storage::block::BlockState;
use itertools::Itertools;


#[derive(Packet, Readable)]
#[packet(0x00, Login)]
pub struct Disconnect {
    pub reason: String,
}


#[derive(EnumReadable, Debug)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x23, Play)]
pub struct JoinGame {
    pub entity_id: u32,
    pub is_hardcore: bool,
    pub game_mode: GameMode,
    // pub previous_game_mode: GameMode,
    // pub world_names: Vec<String>,
    // pub dimension_codec: String,
    // TODO: add rest of data
    // pub entity_id: Int,
    // pub entity_id: Int,
    // pub entity_id: Int,
    // pub entity_id: Int,
    // pub entity_id: Int,
}

#[derive(Packet, Readable)]
#[packet(0x03, Login)]
pub struct SetCompression {
    pub threshold: VarInt
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
    pub uuid: UUIDHyphenated, // 1.16 this is just a raw UUID, 1.12 hyphenated
    pub username: String
}

#[derive(Readable)]
pub struct PlayerPositionAndLookRaw {
    location: Location,
    rotation: Direction,
    flags: BitField,
    teleport_id: VarInt,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x02, Play)]
pub struct SpawnLivingEntity {
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
#[packet(0x27, Play)]
pub struct EntityPosition {
    pub entity_id: VarInt,
    pub loc: ShortLoc,
    pub on_ground: bool,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x36, Play)]
pub struct DestroyEntities {
    pub ids: Vec<VarInt>
}

#[derive(Packet, Debug, Readable)]
#[packet(0x28, Play)]
pub struct EntityPositionAndRot {
    pub entity_id: VarInt,
    pub loc: ShortLoc,
    pub yaw: Angle,
    pub pitch: Angle,
    pub on_ground: bool,
}

#[derive(Packet, Debug, Readable)]
#[packet(0x56, Play)]
pub struct EntityTeleport {
    pub entity_id: VarInt,
    pub location: Location,
    pub yaw: Angle,
    pub pitch: Angle,
    pub on_ground: bool,
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
    pub json: String,
    pub position: u8,
}


#[derive(Packet, Debug, Readable)]
#[packet(0x0b, Play)]
pub struct BlockChange {
    pub location: Position,
    pub block_id: VarInt
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
    pub reason: String
}

#[derive(Packet)]
#[packet(0x20, Play)]
pub struct ChunkColumnPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub column: ChunkColumn
}

pub struct ChunkSection {
    palette: crate::storage::chunk::Palette,
    block_light: [u8; 2048],

    // TODO: fix overworld
    sky_light: Option<[u8; 2048]>,
}

impl ByteReadable for ChunkSection {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let bits_per_block: u8 = byte_reader.read();
        let palette=if bits_per_block <= 8 {
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
        let sky_light = byte_reader.read();
        ChunkSection {
            palette,
            block_light,
            sky_light: Some(sky_light)
        }
    }
}

impl ByteReadable for ChunkColumnPacket {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let chunk_x = byte_reader.read();
        let chunk_z = byte_reader.read();
        let ground_up_continuous: bool = byte_reader.read();
        let VarUInt(mut primary_bitmask) = byte_reader.read();
        let _size: VarUInt = byte_reader.read();

        const INIT: Option<HighMemoryChunkSection> = None;
        let mut sections = [INIT; 16];

        let mut idx = 0;
        while primary_bitmask != 0 {
            if primary_bitmask == 1 {
                let section: ChunkSection = byte_reader.read();
                sections[idx] = Some(HighMemoryChunkSection::new(section.palette));
            }
            primary_bitmask <<= 1;
            idx+=1;
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
            column
        }
    }
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
