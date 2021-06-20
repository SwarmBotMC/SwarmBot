use packets::*;
use packets::types::{VarInt};
use packets::write::{ByteWritable, ByteWriter};
use crate::types::{Location, Direction};

#[derive(Packet, Writable)]
#[packet(0x00, Handshake)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub host: String,

    /// hostname or IP
    pub port: u16,

    /// default 25565
    pub next_state: HandshakeNextState, // 1 for status, 2 for login
}

#[derive(Copy, Clone, EnumWritable)]
#[repr(i32)]
pub enum HandshakeNextState {

    #[deprecated]
    Invalid,

    Status,
    Login,
}

#[derive(Debug, Packet, Writable)]
#[packet(0x00, Login)]
pub struct LoginStart {
    /// player's username
    pub username: String,
}

#[derive(Packet, Writable)]
#[packet(0x01, Login)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

/// Respawning and show stats
#[derive(Writable, Packet)]
#[packet(0x03, Play)]
pub struct ClientStatus {
    pub(crate) action: ClientStatusAction,
}


#[derive(Writable, Packet)]
#[packet(0x0d, Play)]
pub struct PlayerPosition {
    /// True if the client is on the ground, false otherwise.
    pub location: Location,
    pub on_ground: bool,
}

#[derive(Writable, Packet)]
#[packet(0x13, Play)]
pub struct PlayerPositionAndRotation {
    location: Location,
    direction: Direction,
    on_ground: bool,
}

#[derive(Writable, Packet)]
#[packet(0x02, Play)]
pub struct ChatMessage {
    pub message: String
}

impl ChatMessage {
    pub fn message(message: &str) -> ChatMessage {
        ChatMessage {
            message: message.to_string()
        }
    }
}

#[derive(Writable, Packet)]
#[packet(0x25, Play)]
pub struct HeldItemChangeSb {
    pub slot: u16,
}

// pub enum DigStatus {
//     StartDig,
//     CancelDig,
//     FinishDig,
//     DropItemStack,
//     DropItem,
//     ShootArrowFinishEat,
//     SwapItem,
// }
//
// #[derive(Writable, Packet)]
// #[packet(0x1b, Play)]
// pub struct PlayerDigging {
//     pub status: DigStatus,
//
// }

#[derive(Writable, Packet)]
#[packet(0x14, Play)]
pub struct PlayerRotation {
    pub(crate) direction: Direction,
    pub(crate) on_ground: bool,
}

#[derive(Writable, Packet)]
#[packet(0x2c, Play)]
pub struct ArmAnimation {
    pub hand: Hand,
}

#[derive(Writable, Packet)]
#[packet(0x1c, Play)]
pub struct EntityAction {
    /// player id
    pub entity_id: VarInt,
    pub action: Action,
    pub jump_boost: VarInt,
}

#[derive(Writable, Packet)]
#[packet(0x15, Play)]
pub struct PlayerMovement {
    /// True if the client is on the ground, false otherwise.
    on_ground: bool,
}

#[derive(Writable, Packet)]
#[packet(0x16, Play)]
pub struct VehicleMove {
    location: Location,
    direction: Direction,
}

#[repr(i32)]
#[derive(EnumWritable)]
pub enum ClientStatusAction {
    Respawn = 0,
    Stats = 1,
}

#[derive(Writable, Packet)]
#[packet(0x00, Play)]
pub struct TeleportConfirm {
    pub teleport_id: VarInt,
}


#[derive(Writable, Packet)]
#[packet(0x0b, Play)]
pub struct KeepAlive {
    pub id: u64,
}


#[derive(EnumWritable, Debug)]
pub enum Hand {
    Main,
    Off,
}

#[derive(EnumWritable, Debug)]
pub enum Action {
    SneakStart,
    SneakStop,
    LeaveBed,
    SprintStart,
    SprintStop,
    JumpHorseStart,
    JumpHorseStop,
    HorseInvOpen,
    ElytraFlyStart,
}

#[derive(Debug, AdtWritable)]
#[repr(i32)]
pub enum InteractEntityKind {
    Interact {
        target_x: f32,
        target_y: f32,
        target_z: f32,
        hand: Hand,
    },
    Attack,
    InteractAt {
        hand: Hand
    },
}

#[derive(Writable, Packet)]
#[packet(0x0e, Play)]
pub struct InteractEntity {
    pub id: VarInt,
    pub kind: InteractEntityKind,
    pub sneaking: bool,
}
