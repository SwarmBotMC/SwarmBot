use interfaces::types::BlockLocation;
use swarm_bot_packets::types::UUID;

use crate::{
    bootstrap::BotConnection,
    client::{
        pathfind::moves::CardinalDirection, processor::InterfaceIn,
        state::local::inventory::ItemStack,
    },
    error::Res,
    types::{Direction, Location},
};

pub mod v340;

mod encrypt;
mod io;
mod transform;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Mine {
    Start,
    Cancel,
    Finished,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
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
            CardinalDirection::North => Face::PosX,
            CardinalDirection::South => Face::NegX,
            CardinalDirection::West => Face::PosZ,
            CardinalDirection::East => Face::NegZ,
        }
    }
}

impl Face {
    pub fn is_x(self) -> bool {
        matches!(self, Face::PosX | Face::NegX)
    }

    pub fn is_y(self) -> bool {
        matches!(self, Face::PosY | Face::NegY)
    }

    pub fn is_z(self) -> bool {
        matches!(self, Face::PosZ | Face::NegZ)
    }
    pub fn change(self) -> BlockLocation {
        match self {
            Face::NegY => BlockLocation::new(0, -1, 0),
            Face::PosY => BlockLocation::new(0, 1, 0),
            Face::NegZ => BlockLocation::new(0, 0, -1),
            Face::PosZ => BlockLocation::new(0, 0, 1),
            Face::NegX => BlockLocation::new(-1, 0, 0),
            Face::PosX => BlockLocation::new(1, 0, 0),
        }
    }
}

impl From<u8> for Face {
    fn from(elem: u8) -> Self {
        use Face::*;
        match elem {
            0 => NegY,
            1 => PosY,
            2 => NegZ,
            3 => PosZ,
            4 => NegX,
            5 => PosX,
            _ => panic!("invalid"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left,
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

pub enum InvAction {
    Q(u16),
    // drop full stack
    CtrlQ(u16),
    Click(u16, MouseButton, ItemStack),
    ShiftClick(u16, MouseButton, ItemStack),
}

pub trait InterfaceOut {
    fn place_block(&mut self, against: BlockLocation, face: Face);
    fn attack_entity(&mut self, id: u32);
    fn send_chat(&mut self, message: &str);
    fn inventory_action(&mut self, action: InvAction);
    fn swing_arm(&mut self);
    fn finish_eating(&mut self);

    /// default right click https://wiki.vg/index.php?title=Protocol&oldid=14204#Chunk_Data
    fn use_item(&mut self);

    fn change_slot(&mut self, number: u8);
    fn mine(&mut self, location: BlockLocation, mine: Mine, face: Face);
    fn respawn(&mut self);
    fn teleport(&mut self, location: Location);
    fn look(&mut self, direction: Direction);
    fn teleport_and_look(&mut self, location: Location, direction: Direction, on_ground: bool);
}

#[async_trait::async_trait]
pub trait Minecraft: Sized {
    type Queue: EventQueue;
    type Interface: InterfaceOut;
    async fn login(conn: BotConnection) -> Res<Login<Self::Queue, Self::Interface>>;
}

pub trait EventQueue {
    fn flush(&mut self, processor: &mut impl InterfaceIn);
}

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}

/// login for a given bot. Holds
pub struct Login<E: EventQueue, I: InterfaceOut> {
    pub queue: E,
    pub out: I,
    pub info: ClientInfo,
}
