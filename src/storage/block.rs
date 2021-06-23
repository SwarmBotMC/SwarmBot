use std::fmt::{Display, Formatter};
use crate::types::Location;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct BlockState(pub u32);


#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Default)]
pub struct BlockLocation(pub i64, pub i64, pub i64);

impl BlockLocation {
    pub fn centered(&self) -> Location {
        Location {
            x: self.0 as f64 + 0.5,
            y: self.1 as f64,
            z: self.2 as f64 + 0.5
        }
    }
}

// trait IterDirection {
//     type Item;
//     type IntoIter: Iterator<Item=Self::Item>;
//     fn iter_y(&self, pos: bool) -> Self::IntoIter;
//     fn iter_x(&self, pos: bool) -> Self::IntoIter;
//     fn iter_z(&self, pos: bool) -> Self::IntoIter;
// }
//
// impl IterDirection for Range<BlockLocation> {
//     type Item = BlockLocation;
//     type IntoIter = ();
//
//     fn iter_y(&self, pos: bool) -> Self::IntoIter {
//         todo!()
//     }
//
//     fn iter_x(&self, pos: bool) -> Self::IntoIter {
//         todo!()
//     }
//
//     fn iter_z(&self, pos: bool) -> Self::IntoIter {
//         todo!()
//     }
// }

enum Priority {
    X,
    Y,
    Z,
}

struct BlockIter {
    from: BlockLocation,
    to: BlockLocation,
    idx: usize,
    cross_section: usize,
    d_second: usize,
    dx: usize,
    dy: usize,
    dz: usize,
    size: usize,
    pos: bool,
    priority: Priority,
}

// impl BlockIter {
//     fn new(pos: bool, mut from: BlockLocation, mut to: BlockLocation, priority: Priority) -> BlockIter {
//         let dx = to.0 - from.0 + 1;
//         let dy = to.1 - from.1 + 1;
//         let dz = to.2 - from.2 + 1;
//     }
// }

impl Iterator for BlockIter {
    type Item = BlockLocation;

    fn next(&mut self) -> Option<Self::Item> {
        let mut on = self.idx;
        if on < self.size {
            let first = (self.idx / self.cross_section) as i64;
            let left_over = self.idx % self.cross_section;
            let second = (left_over / self.d_second) as i64;
            let third = (left_over % self.d_second) as i64;

            let BlockLocation(x,y,z) = self.from;

            let block_loc = match self.priority {
                Priority::X => {
                    BlockLocation(x + first,y + second,z + third)
                }
                Priority::Y => {
                    BlockLocation(x+ third,y + first,z + second)
                }
                Priority::Z => {
                    BlockLocation(x+ third,y + second,z + first)
                }
            };
            on += 1;
            Some(block_loc)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for BlockIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl BlockLocation {
    pub(crate) fn dist2(&self, other: BlockLocation) -> i64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        let dz = self.2 - other.2;
        dx * dx + dy * dy + dz * dz
    }

    pub(crate) fn dist(&self, other: BlockLocation) -> f64 {
        (self.dist2(other) as f64).sqrt()
    }
}

impl Display for BlockLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}, {}]", self.0, self.1, self.2))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BlockApprox {
    Realized(BlockState),
    Estimate(SimpleType),
}

pub const AIR: BlockApprox = BlockApprox::Estimate(SimpleType::WalkThrough);

impl BlockApprox {
    pub fn s_type(&self) -> SimpleType {
        match self {
            BlockApprox::Realized(x) => {
                x.simple_type()
            }
            BlockApprox::Estimate(x) => *x
        }
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

impl BlockState {
    pub fn id(&self) -> u32 {
        self.0 >> 4
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
            1..=5 |7 | 12..=25 | 29 | 33 |35 | 41 ..=43 | 45..=49 | 52 | 56..=58 | 60..=62 | 73 | 74
        )
    }

    pub fn is_water(&self) -> bool {
        matches!(self.id(), 8 | 9)
    }

    pub fn walk_through(&self) -> bool {
        self.is_water() || self.no_motion_effect()
    }

    pub fn no_motion_effect(&self) -> bool {
        matches!(self.id(), 0 | 6)
    }
}
