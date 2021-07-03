/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::bootstrap::blocks::BlockData;
use crate::client::state::inventory::ItemStack;
use crate::storage::block::BlockKind;
use crate::types::Enchantment;

#[derive(Copy, Clone)]
pub enum Material {
    Hand,
    Wood,
    Stone,
    Iron,
    Diamond,
    Gold,
}


#[derive(Copy, Clone)]
pub enum ToolKind {
    Generic,
    Pickaxe,
    Hoe,
    Shovel,
    Axe,
    Sword,
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
    pub kind: ToolKind,
    pub enchantments: Vec<Enchantment>
}


impl From<&ItemStack> for Tool {
    fn from(stack: &ItemStack) -> Self {
        use crate::client::physics::tools::ToolKind::*;
        use crate::client::physics::tools::Material::*;

        let id = stack.kind.id();

        let mut simple_tool = match id {
            256 => Tool::simple(Shovel, Iron),
            257 => Tool::simple(Pickaxe, Iron),
            258 => Tool::simple(Axe, Iron),

            269 => Tool::simple(Shovel, Wood),
            270 => Tool::simple(Pickaxe, Wood),
            271 => Tool::simple(Axe, Wood),

            273 => Tool::simple(Shovel, Stone),
            274 => Tool::simple(Pickaxe, Stone),
            275 => Tool::simple(Axe, Stone),

            277 => Tool::simple(Shovel, Diamond),
            278 => Tool::simple(Pickaxe, Diamond),
            279 => Tool::simple(Axe, Diamond),

            284 => Tool::simple(Shovel, Gold),
            285 => Tool::simple(Pickaxe, Gold),
            286 => Tool::simple(Axe, Gold),

            _ => Tool::simple(Generic, Hand)
        };

        if let Some(nbt) = stack.nbt.as_ref() {
            simple_tool.enchantments = nbt.ench.clone()
        }

        simple_tool
    }
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            material: Material::Hand,
            kind: ToolKind::Generic,
            enchantments: vec![]
        }
    }
}

impl Tool {
    pub fn simple(kind: ToolKind, material: Material) -> Self {
        Self { material, kind, enchantments: Vec::new() }
    }

    pub fn efficiency(&self) -> Option<u16> {
        self.enchantments.iter().filter_map(|ench| ench.efficiency())
            .max()
    }

    fn strength_against_block(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData) -> f64 {

        let hardness = kind.hardness(data).unwrap_or(f64::INFINITY);
        if hardness < 0.0 { return 0.0; }

        let mut d = self.material.strength();

        let efficiency = self.efficiency().unwrap_or(0);

        if efficiency > 0 {
            d += (efficiency.pow(2) + 1) as f64
        }

        if underwater { d /= 5.0; }
        if !on_ground { d /= 5.0; }

        d / hardness / 30.0
    }

    /// https://minecraft.fandom.com/wiki/Breaking#Speed
    pub fn wait_time(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData) -> usize {
        let strength = self.strength_against_block(kind, underwater, on_ground, data);
        (1.0 / strength).round() as usize
    }
}
