/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::fmt::{Display, Formatter};
use std::ops::{Add, Index};

use crate::bootstrap::blocks::BlockData;
use crate::types::Location;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct BlockKind(u32);

impl From<u32> for BlockKind {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl BlockKind {
    pub const DEFAULT_SLIP: f64 = 0.6;

    pub const LADDER: BlockKind = BlockKind(65);

    pub fn hardness(&self, blocks: &BlockData) -> Option<f64> {
        blocks.by_id(self.0).unwrap().hardness
    }

    pub fn slip(&self) -> f64 {
        match self.0 {
            266 => 0.989, // blue ice
            79 | 174 | 212 => 0.98, // ice, packed ice, or frosted ice
            37 => 0.8, // slime block
            _ => Self::DEFAULT_SLIP
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct BlockState(pub u32);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);
    pub const STONE: BlockState = BlockState(16);

    pub fn from(id: u32, data: u32) -> BlockState {
        BlockState((id << 4) + data)
    }

    pub fn id(&self) -> u32 {
        self.0 >> 4
    }

    pub fn kind(&self) -> BlockKind {
        BlockKind(self.id())
    }

    pub fn simple_type(&self) -> SimpleType {
        if self.full_block() {
            return SimpleType::Solid;
        }

        if self.is_water() {
            return SimpleType::Water;
        }

        if self.walk_through() {
            return SimpleType::WalkThrough;
        }

        SimpleType::Avoid
    }

    pub fn metadata(&self) -> u8 {
        (self.0 & 0b1111) as u8
    }

    pub fn full_block(&self) -> bool {
        //consider 54 |
        matches!(self.id(),
            1..=5 |7 | 12..=25 | 29 | 33 |35 | 41 ..=43 | 45..=49 | 52 | 56..=58 | 60..=62 | 73 | 74 |
            78..=80| // snow, ice
            82| // clay
            84|86|87|89|91|95|
            97| // TODO: avoid this is a monster egg
            98..=100|
            // TODO: account panes
            103|110|112|118|121|123..=125|
            129|133|137..=138|155|159|161|162|
            165| // TODO: slime block special fall logic
            166|
            168..=170| // TODO: special haybale logic
            172..=174|
            179|181|199..=202|
            204|206|208..=212|214..=255

        )
    }

    pub fn is_water(&self) -> bool {
        matches!(self.id(), 8 | 9 |
            65 // ladder ... this is VERY jank
        )
    }

    pub fn walk_through(&self) -> bool {
        self.is_water() || self.no_motion_effect()
    }

    pub fn no_motion_effect(&self) -> bool {
        matches!(self.id(),
            0| // air
            6|// sapling
            27|28| //  rail
            31| // grass/fern/dead shrub
            38|37|// flower
            39|40| //mushroom
            50|//torch
            59|// wheat
            66|68|69|70|72|75|76|77|83|
            90| // portal
            104|105|106|
            115|119|
            175..=177




        )
    }
}


/// A block location stored by (x,z) = i32, y = i16. y is signed to preserve compatibility with 1.17, where the world
/// height can be much higher and goes to negative values.
#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Default)]
pub struct BlockLocation {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl Add for BlockLocation {
    type Output = BlockLocation;

    fn add(self, rhs: Self) -> Self::Output {
        let BlockLocation { x, y, z } = self;
        BlockLocation::new(x + rhs.x, y + rhs.y, z + rhs.z)
    }
}

impl BlockLocation {

    pub fn new(x: i32, y: i16, z: i32) -> BlockLocation {
        BlockLocation { x, y, z }
    }

    pub fn below(&self) -> BlockLocation {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z
        }
    }

    pub fn get(&self, idx: usize) -> i32 {
        match idx {
            0 => self.x,
            1 => self.y as i32,
            2 => self.z,
            _ => panic!("invalid index for block location")
        }
    }

    pub fn set(&mut self, idx: usize, value: i32) {
        match idx {
            0 => self.x = value,
            1 => self.y = value as i16,
            2 => self.z = value,
            _ => panic!("invalid index for block location")
        }
    }


    pub fn from_flts(x: impl num::Float, y: impl num::Float, z: impl num::Float) -> BlockLocation {
        let x = num::cast(x.floor()).unwrap();
        let y = num::cast(y.floor()).unwrap();
        let z = num::cast(z.floor()).unwrap();
        BlockLocation::new(x, y, z)
    }

    pub fn add_y(&self, dy: i16) -> BlockLocation {
        let &BlockLocation { x, y, z } = self;
        Self { x, y: y + dy, z }
    }

    pub fn center_bottom(&self) -> Location {
        Location {
            x: self.x as f64 + 0.5,
            y: self.y as f64,
            z: self.z as f64 + 0.5,
        }
    }
}

impl BlockLocation {
    pub(crate) fn dist2(&self, other: BlockLocation) -> u64 {
        let dx = (self.x - other.x).abs() as u64;
        let dy = (self.y - other.y).abs() as u64;
        let dz = (self.z - other.z).abs() as u64;
        dx * dx + dy * dy + dz * dz
    }

    pub(crate) fn manhatten(&self, other: BlockLocation) -> u64 {
        let dx = (self.x - other.x).abs() as u64;
        let dy = (self.y - other.y).abs() as u64;
        let dz = (self.z - other.z).abs() as u64;
        dx + dy + dz
    }

    pub(crate) fn dist(&self, other: BlockLocation) -> f64 {
        (self.dist2(other) as f64).sqrt()
    }
}

impl Display for BlockLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}, {}]", self.x, self.y, self.z))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BlockApprox {
    Realized(BlockState),
    Estimate(SimpleType),
}


impl BlockApprox {
    pub const AIR: BlockApprox = BlockApprox::Estimate(SimpleType::WalkThrough);

    pub fn s_type(&self) -> SimpleType {
        match self {
            BlockApprox::Realized(x) => {
                x.simple_type()
            }
            BlockApprox::Estimate(x) => *x
        }
    }

    pub fn as_real(&self) -> BlockState {
        match self {
            BlockApprox::Realized(inner) => *inner,
            _ => panic!("was not relized")
        }
    }

    pub fn is_solid(&self) -> bool {
        self.s_type() == SimpleType::Solid
    }

    pub fn is_walkable(&self) -> bool {
        self.s_type() == SimpleType::WalkThrough
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SimpleType {
    Solid,
    Water,
    Avoid,
    WalkThrough,
}

impl SimpleType {
    pub fn id(&self) -> u8 {
        match self {
            SimpleType::Solid => 0,
            SimpleType::Water => 1,
            SimpleType::Avoid => 2,
            SimpleType::WalkThrough => 3
        }
    }
}

impl From<u8> for SimpleType {
    fn from(id: u8) -> Self {
        match id {
            0 => SimpleType::Solid,
            1 => SimpleType::Water,
            2 => SimpleType::Avoid,
            3 => SimpleType::WalkThrough,
            _ => panic!("invalid id")
        }
    }
}
