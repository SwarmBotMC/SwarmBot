/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::bootstrap::block_data::{BlockData, Material};
use crate::storage::block::BlockKind;
use crate::types::Enchantment;
use crate::client::state::local::inventory::ItemStack;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ToolMat {
    Hand,
    Wood,
    Stone,
    Iron,
    Diamond,
    Gold,
}


#[derive(Copy, Clone, Debug)]
pub enum ToolKind {
    Generic,
    Pickaxe,
    Hoe,
    Shovel,
    Axe,
    Sword,
}


impl ToolMat {
    pub fn strength(self) -> f64 {
        match self {
            ToolMat::Hand => 1.0,
            ToolMat::Wood => 2.0,
            ToolMat::Stone => 4.0,
            ToolMat::Iron => 6.0,
            ToolMat::Diamond => 8.0,
            ToolMat::Gold => 12.0,
        }
    }
}

#[derive(Debug)]
pub struct Tool {
    pub material: ToolMat,
    pub kind: ToolKind,
    pub id: u32,
    pub enchantments: Vec<Enchantment>,
}


impl From<&ItemStack> for Tool {
    fn from(stack: &ItemStack) -> Self {
        use crate::client::physics::tools::ToolKind::*;
        use crate::client::physics::tools::ToolMat::*;

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

        simple_tool.id = id;

        if let Some(nbt) = stack.nbt.as_ref() {
            simple_tool.enchantments = nbt.ench.clone()
        }

        simple_tool
    }
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            material: ToolMat::Hand,
            kind: ToolKind::Generic,
            enchantments: vec![],
            id: 0,
        }
    }
}

impl Tool {
    pub fn simple(kind: ToolKind, material: ToolMat) -> Self {
        Self { material, kind, enchantments: Vec::new(), id: 0 }
    }

    pub fn efficiency(&self) -> Option<u16> {
        self.enchantments.iter().filter_map(|ench| ench.efficiency())
            .max()
    }

    fn strength_against_block(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData) -> f64 {
        let block = kind.data(data);

        let can_harvest = match block.material {
            Material::Web | Material::Generic | Material::Plant | Material::Dirt | Material::Wool | Material::Wood => true,
            Material::Rock => block.harvest_tools.contains(&self.id)
        };

        let best_tool = match (block.material, self.kind) {
            (Material::Rock, ToolKind::Pickaxe) => true,
            (Material::Wood, ToolKind::Axe) => true,
            (Material::Dirt, ToolKind::Shovel) => true,
            (Material::Web, ToolKind::Sword) => true,
            (Material::Plant, _) => false, // need sheers
            (Material::Generic, _) => false, // i.e., glass

            _ => false
        };

        let hardness = block.hardness.unwrap_or(f64::INFINITY).max(0.0);

        let mut d = 1.0;


        if best_tool {
            if can_harvest {
                d *= self.material.strength()
            }

            let efficiency = self.efficiency().unwrap_or(0);

            if efficiency > 0 {
                d += (efficiency.pow(2) + 1) as f64
            }
        }

        if underwater { d /= 5.0; }
        if !on_ground { d /= 5.0; }


        let res = d / hardness;

        if can_harvest {
            res / 30.
        } else {
            res / 100.
        }
    }

    /// https://minecraft.fandom.com/wiki/Breaking#Speed
    pub fn wait_time(&self, kind: BlockKind, underwater: bool, on_ground: bool, data: &BlockData) -> usize {
        let strength = self.strength_against_block(kind, underwater, on_ground, data);
        (1.0 / strength).round() as usize
    }
}


#[cfg(test)]
mod tests {
    use crate::bootstrap::block_data::BlockData;
    use crate::client::physics::tools::{Tool, ToolKind, ToolMat};
    use crate::storage::block::BlockKind;

    #[test]
    fn test_break_time() {
        let data = BlockData::read().unwrap();

        let mut diamond_pick = Tool::simple(ToolKind::Pickaxe, ToolMat::Diamond);
        diamond_pick.id = 278;

        let mut diamond_shovel = Tool::simple(ToolKind::Shovel, ToolMat::Diamond);
        diamond_shovel.id = 277;

        let hand = Tool::simple(ToolKind::Generic, ToolMat::Hand);

        let time = |tool: &Tool, kind: BlockKind| tool.wait_time(kind, false, true, &data);

        // glass
        assert_eq!(9, time(&hand, BlockKind::GLASS));
        assert_eq!(9, time(&diamond_pick, BlockKind::GLASS));
        assert_eq!(9, time(&diamond_shovel, BlockKind::GLASS));


        // stone
        assert_eq!(150, time(&hand, BlockKind::STONE));
        assert_eq!(6, time(&diamond_pick, BlockKind::STONE));
        assert_eq!(150, time(&diamond_shovel, BlockKind::STONE));

        // dirt
        assert_eq!(15, time(&hand, BlockKind::DIRT));
        assert_eq!(15, time(&diamond_pick, BlockKind::DIRT));
        assert_eq!(2, time(&diamond_shovel, BlockKind::DIRT));

        // leaves
        assert_eq!(6, time(&hand, BlockKind::LEAVES));
        assert_eq!(6, time(&diamond_pick, BlockKind::LEAVES));
        assert_eq!(6, time(&diamond_shovel, BlockKind::LEAVES));
    }
}
