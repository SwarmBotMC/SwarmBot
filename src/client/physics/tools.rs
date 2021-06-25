use crate::storage::block::{BlockState, BlockKind};
use crate::bootstrap::blocks::BlockData;

pub struct Material {
    strength: f64,
}

impl Material {
    pub const WOOD: Self = Self::new(2.0);
    pub const STONE: Self = Self::new(4.0);
    pub const IRON: Self = Self::new(6.0);
    pub const DIAMOND: Self = Self::new(8.0);
    pub const GOLD: Self = Self::new(12.0);

    const fn new(strength: f64) -> Material {
        Self { strength }
    }
}

pub struct Tool {
    material: Material,
}

impl Tool {
    pub fn new(material: Material ) -> Self {
        Self{material}
    }
    fn strength_against_block(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData) -> f64 {
        let hardness = kind.hardness(data).unwrap_or(f64::INFINITY);
        if hardness < 0.0 { return 0.0; }

        let mut d = 1.0;
        d *= self.material.strength;
        if underwater { d /= 5.0; }
        if !on_ground { d /= 5.0; }

        d / hardness / 30.0
    }

    pub fn wait_time(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData) -> usize {
        let strength = self.strength_against_block(kind,underwater, on_ground, data);
        (1.0 / strength).round() as usize
    }
}
