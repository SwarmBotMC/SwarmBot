/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::bootstrap::blocks::BlockData;
use crate::storage::block::BlockKind;

pub struct Material {
    strength: f64,
}

impl Material {
    pub const HAND: Self = Self::new(1.0);
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
    pub fn new(material: Material) -> Self {
        Self { material }
    }
    fn strength_against_block(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData, efficiency: u32) -> f64 {
        let hardness = kind.hardness(data).unwrap_or(f64::INFINITY);
        if hardness < 0.0 { return 0.0; }

        let mut d = self.material.strength;

        if efficiency > 0 {
            d += (efficiency.pow(2) + 1) as f64
        }

        if underwater { d /= 5.0; }
        if !on_ground { d /= 5.0; }

        d / hardness / 30.0
    }

    /// https://minecraft.fandom.com/wiki/Breaking#Speed
    pub fn wait_time(&self, kind: BlockKind, underwater: bool, on_ground: bool, efficiency: u32, data: &BlockData) -> usize {
        let strength = self.strength_against_block(kind, underwater, on_ground, data, efficiency);
        (1.0 / strength).round() as usize
    }
}
