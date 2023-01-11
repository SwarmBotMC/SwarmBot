use swarm_bot_packets::{
    types::VarInt,
    write::{ByteWritable, ByteWriter},
    AdtWritable, EnumWritable, Packet, Writable,
};

use crate::types::{Direction, Location, Position, Slot};

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
#[allow(dead_code)]
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
#[packet(0x0e, Play)]
pub struct PlayerPositionAndRotation {
    pub location: Location,
    pub direction: Direction,
    pub on_ground: bool,
}

#[derive(Packet, Writable)]
#[packet(0x20, Login)]
pub struct UseItem {
    pub(crate) hand: Hand,
}

#[derive(Writable, Packet)]
#[packet(0x02, Play)]
pub struct ChatMessage {
    pub message: String,
}

#[derive(Writable, Packet)]
#[packet(0x07, Play)]
pub struct ClickWindow {
    pub window_id: u8,
    pub slot: u16,
    pub button: u8,
    pub action_number: u16,
    pub mode: VarInt,
    pub clicked: Slot,
}

#[derive(Writable, Default)]
pub struct BlockCursor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Writable, Packet)]
#[packet(0x1f, Play)]
pub struct PlaceBlock {
    pub location: Position,
    pub face: VarInt,
    pub hand: Hand,
    pub cursor: BlockCursor,
}

#[derive(Writable, Packet)]
#[packet(0x25, Play)]
pub struct HeldItemChangeSb {
    pub slot: u16,
}

#[derive(Writable, Packet)]
#[packet(0x0f, Play)]
pub struct PlayerLook {
    pub(crate) direction: Direction,
    pub(crate) on_ground: bool,
}

#[derive(Writable, Packet)]
#[packet(0x1d, Play)]
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

#[derive(EnumWritable, Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DigStatus {
    Started,
    Cancelled,
    Finished,
    DropItemStack,
    DropItem,
    ShootArrowOrFinishEat,
    SwapItem, // location 0,0,0 face-y
}

#[derive(Writable, Packet)]
#[packet(0x14, Play)]
pub struct PlayerDig {
    pub status: DigStatus,
    pub position: Position,
    pub face: u8,
}

impl PlayerDig {
    pub fn status(status: DigStatus) -> Self {
        Self {
            status,
            position: Position::default(),
            face: 0,
        }
    }
}

pub type ChangeSlot = HeldItemChange;

#[derive(Writable, Packet)]
#[packet(0x1a, Play)]
pub struct HeldItemChange {
    pub slot: u16,
}

#[repr(i32)]
#[derive(EnumWritable)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum Hand {
    Main,
    Off,
}

#[derive(EnumWritable, Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum InteractEntityKind {
    Interact {
        target_x: f32,
        target_y: f32,
        target_z: f32,
        hand: Hand,
    },
    Attack,
    InteractAt {
        hand: Hand,
    },
}

#[derive(Writable, Packet)]
#[packet(0x0a, Play)]
pub struct InteractEntity {
    pub id: VarInt,
    pub kind: InteractEntityKind,
}
