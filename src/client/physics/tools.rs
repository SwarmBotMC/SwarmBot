/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::bootstrap::blocks::BlockData;
use crate::storage::block::BlockKind;

#[derive(Copy, Clone)]
pub enum Material {
    Hand,
    Wood,
    Stone,
    Iron,
    Diamond,
    Gold
}


#[derive(Copy, Clone)]
pub enum ToolKind {
    Generic,
    Pickaxe,
    Hoe,
    Shovel,
    Axe,
    Sword
}


impl Material {

    pub fn strength(self) -> f64 {
        match self {
            Material::Hand => 1.0,
            Material::Wood => 2.0,
            Material::Stone => 4.0,
            Material::Iron => 6.0,
            Material::Diamond => 8.0,
            Material::Gold => 12.0,
        }
    }
}

pub struct Tool {
    pub material: Material,
    pub kind: ToolKind
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            material: Material::Hand,
            kind: ToolKind::Generic
        }
    }
}

impl Tool {
    pub fn new(kind: ToolKind, material: Material) -> Self {
        Self { material, kind }
    }
    fn strength_against_block(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData, efficiency: u32) -> f64 {
        let hardness = kind.hardness(data).unwrap_or(f64::INFINITY);
        if hardness < 0.0 { return 0.0; }

        let mut d = self.material.strength();

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
