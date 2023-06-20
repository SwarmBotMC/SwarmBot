use async_trait::async_trait;
use interfaces::types::BlockLocation;
use swarm_bot_packets::types::UUID;

use crate::{
    bootstrap::BotConnection,
    client::{
        pathfind::moves::CardinalDirection, processor::InterfaceIn,
        state::local::inventory::ItemStack,
    },
    types::{Direction, Location},
};

pub mod v340;

mod encrypt;
mod io;
mod transform;

/// TODO
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum Mine {
    Start,
    #[allow(unused)]
    Cancel,
    Finished,
}

/// The face. This is usually used in combination with a block.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum Face {
    NegY,
    PosY,
    NegZ,
    PosZ,
    NegX,
    PosX,
}

impl From<CardinalDirection> for Face {
    fn from(dir: CardinalDirection) -> Self {
        match dir {
            CardinalDirection::North => Self::PosX,
            CardinalDirection::South => Self::NegX,
            CardinalDirection::West => Self::PosZ,
            CardinalDirection::East => Self::NegZ,
        }
    }
}

impl Face {
    /// If is on the x-axis
    #[allow(unused)]
    pub const fn is_x(self) -> bool {
        matches!(self, Self::PosX | Self::NegX)
    }

    /// if is on the y-axis
    #[allow(unused)]
    pub const fn is_y(self) -> bool {
        matches!(self, Self::PosY | Self::NegY)
    }

    /// if is on z-axis
    #[allow(unused)]
    pub const fn is_z(self) -> bool {
        matches!(self, Self::PosZ | Self::NegZ)
    }

    /// get the unit location of each face
    pub fn unit_location(self) -> BlockLocation {
        match self {
            Self::NegY => BlockLocation::new(0, -1, 0),
            Self::PosY => BlockLocation::new(0, 1, 0),
            Self::NegZ => BlockLocation::new(0, 0, -1),
            Self::PosZ => BlockLocation::new(0, 0, 1),
            Self::NegX => BlockLocation::new(-1, 0, 0),
            Self::PosX => BlockLocation::new(1, 0, 0),
        }
    }
}

impl From<u8> for Face {
    #[allow(clippy::panic)]
    fn from(elem: u8) -> Self {
        match elem {
            0 => Self::NegY,
            1 => Self::PosY,
            2 => Self::NegZ,
            3 => Self::PosZ,
            4 => Self::NegX,
            5 => Self::PosX,
            _ => panic!("invalid"),
        }
    }
}

/// which mouse button we are press
#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    /// left click
    Left,
    /// right click
    Right,
}

impl From<MouseButton> for u8 {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
        }
    }
}

/// the inventory action we are doing
pub enum InvAction {
    /// pressing the 'Q' key
    #[allow(unused)]
    Q(u16),
    /// drop full stack
    CtrlQ(u16),
    /// click with a given [`MouseButton`] on a specific slot carrying an
    /// [`ItemStack`]
    #[allow(unused)]
    Click(u16, MouseButton, ItemStack),
    /// shift click with a given [`MouseButton`] on a specific slot an
    /// [`ItemStack`]
    #[allow(unused)]
    ShiftClick(u16, MouseButton, ItemStack),
}

/// the interface that we can use to send packets (do actions)
pub trait InterfaceOut {
    /// place a block against `against` on `face`
    fn place_block(&mut self, against: BlockLocation, face: Face);

    /// attack an entity with id {id}
    fn attack_entity(&mut self, id: u32);

    /// send a given chat message. This can be formatting using [Minecraft formatting codes](https://minecraft.fandom.com/wiki/Formatting_codes)
    fn send_chat(&mut self, message: &str);

    /// perform an inventory action
    fn inventory_action(&mut self, action: InvAction);

    /// swing arm. This is equivalent to elft clicking
    fn swing_arm(&mut self);

    /// finish eating TODO: I think this can be used for mining too maybe>?
    fn finish_eating(&mut self);

    /// default right click <https://wiki.vg/index.php?title=Protocol&oldid=14204#Chunk_Data>
    fn use_item(&mut self);

    /// change to slot `number`. This is equivalent to pressing the `1-9` keys
    /// on PC
    fn change_slot(&mut self, number: u8);

    /// mine a given block
    fn mine(&mut self, location: BlockLocation, mine: Mine, face: Face);

    /// respawn. this can only be used when the player is dead
    fn respawn(&mut self);

    /// teleport to a given location. This is used for walking/running. If the
    /// values are too extreme, the player will most likely get kicked from
    /// the server for hacking
    fn teleport(&mut self, location: Location);

    /// look in a given [`Direction`]
    fn look(&mut self, direction: Direction);

    /// teleport to a location and look
    fn teleport_and_look(&mut self, location: Location, direction: Direction, on_ground: bool);
}

/// the minecraft trait. This represents a full Minecraft bot
#[async_trait]
pub trait Minecraft: Sized {
    /// the queue we are receiving from
    type Queue: EventQueue;

    /// the interface we are sending out to
    type Interface: InterfaceOut;

    /// perform a login action to the server given a [`BotConnection`] .
    ///
    /// This is an **async function** and will return the [`Login`] struct if
    /// successful.
    async fn login(conn: BotConnection) -> anyhow::Result<Login<Self::Queue, Self::Interface>>;
}

/// the event queue. Gets all packets that have been received and defines a
/// generic interface through which an [`InterfaceIn`] can access the events.
/// TODO: is this right?
pub trait EventQueue {
    /// flush the queue (process all events)
    fn flush(&mut self, processor: &mut impl InterfaceIn);
}

/// Information about the client
#[derive(Debug)]
pub struct ClientInfo {
    /// The name of the player
    pub username: String,
    /// the UUID of the player
    pub uuid: UUID,

    /// the entity ID that the server gave the player
    pub entity_id: u32,
}

/// login for a given bot. Holds
///
/// - `queue`: An [`EventQueue`] that we can use for processing events
/// - `out`: An interface we can use for sending data to
/// - `info`: The information of the bot that was logged in. This can be used by
///   the [`Runner`]
/// for querying purposes.
pub struct Login<E: EventQueue, I: InterfaceOut> {
    /// The [`EventQueue`]
    pub queue: E,

    /// the [`InterfaceOut`]
    pub out: I,

    /// information about the client
    pub info: ClientInfo,
}
