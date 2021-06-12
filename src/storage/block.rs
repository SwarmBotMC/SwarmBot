#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct BlockState(pub u32);


#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct BlockLocation(pub i64, pub i64, pub i64);

impl BlockLocation {
    pub(crate) fn dist2(&self, other: BlockLocation) -> i64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        let dz = self.2 - other.2;
        dx*dx + dy*dy + dz*dz
    }

    pub(crate) fn dist(&self, other: BlockLocation) -> f64 {
        (self.dist2(other) as f64).sqrt()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BlockApprox {
    Realized(BlockState),
    Estimate(SimpleType)
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
