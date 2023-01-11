use interfaces::types::{
    block_data::{BlockData, Material},
    BlockKind,
};

use crate::{client::state::local::inventory::ItemStack, types::Enchantment};

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
#[allow(unused)]
pub enum ToolKind {
    Generic,
    Pickaxe,
    Hoe,
    Shovel,
    Axe,
    Sword,
}

impl ToolMat {
    pub const fn strength(self) -> f64 {
        match self {
            Self::Hand => 1.0,
            Self::Wood => 2.0,
            Self::Stone => 4.0,
            Self::Iron => 6.0,
            Self::Diamond => 8.0,
            Self::Gold => 12.0,
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
        use crate::client::physics::tools::{
            ToolKind::{Axe, Generic, Pickaxe, Shovel},
            ToolMat::{Diamond, Gold, Hand, Iron, Stone, Wood},
        };

        let id = stack.kind.id();

        let mut simple_tool = match id {
            256 => Self::simple(Shovel, Iron),
            257 => Self::simple(Pickaxe, Iron),
            258 => Self::simple(Axe, Iron),

            269 => Self::simple(Shovel, Wood),
            270 => Self::simple(Pickaxe, Wood),
            271 => Self::simple(Axe, Wood),

            273 => Self::simple(Shovel, Stone),
            274 => Self::simple(Pickaxe, Stone),
            275 => Self::simple(Axe, Stone),

            277 => Self::simple(Shovel, Diamond),
            278 => Self::simple(Pickaxe, Diamond),
            279 => Self::simple(Axe, Diamond),

            284 => Self::simple(Shovel, Gold),
            285 => Self::simple(Pickaxe, Gold),
            286 => Self::simple(Axe, Gold),

            _ => Self::simple(Generic, Hand),
        };

        simple_tool.id = id;

        if let Some(nbt) = stack.nbt.as_ref() {
            simple_tool.enchantments = nbt.ench.clone().unwrap_or_default();
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
    pub const fn simple(kind: ToolKind, material: ToolMat) -> Self {
        Self {
            material,
            kind,
            enchantments: Vec::new(),
            id: 0,
        }
    }

    pub fn efficiency(&self) -> Option<u16> {
        self.enchantments
            .iter()
            .filter_map(|ench| ench.efficiency())
            .max()
    }

    fn strength_against_block(
        &self,
        kind: BlockKind,
        underwater: bool,
        on_ground: bool,
        data: &BlockData,
    ) -> f64 {
        let block = kind.data(data);

        let can_harvest = match block.material {
            Material::Web
            | Material::Generic
            | Material::Plant
            | Material::Dirt
            | Material::Wool
            | Material::Wood => true,
            Material::Rock => block.harvest_tools.contains(&self.id),
        };

        #[allow(clippy::match_same_arms)]
        let best_tool = match (block.material, self.kind) {
            (Material::Rock, ToolKind::Pickaxe) => true,
            (Material::Wood, ToolKind::Axe) => true,
            (Material::Dirt, ToolKind::Shovel) => true,
            (Material::Web, ToolKind::Sword) => true,
            (Material::Plant, _) => false,   // need sheers
            (Material::Generic, _) => false, // i.e., glass

            _ => false,
        };

        let hardness = block.hardness.unwrap_or(f64::INFINITY).max(0.0);

        let mut d = 1.0;

        if best_tool {
            if can_harvest {
                d *= self.material.strength();
            }

            let efficiency = self.efficiency().unwrap_or(0);

            if efficiency > 0 {
                d += f64::from(efficiency.pow(2) + 1);
            }
        }

        if underwater {
            d /= 5.0;
        }
        if !on_ground {
            d /= 5.0;
        }

        let res = d / hardness;

        if can_harvest {
            res / 30.
        } else {
            res / 100.
        }
    }

    /// <https://minecraft.fandom.com/wiki/Breaking#Speed>
    pub fn wait_time(
        &self,
        kind: BlockKind,
        underwater: bool,
        on_ground: bool,
        data: &BlockData,
    ) -> usize {
        let strength = self.strength_against_block(kind, underwater, on_ground, data);
        (1.0 / strength).round() as usize
    }
}

#[cfg(test)]
mod tests {
    use interfaces::types::{block_data::BlockData, BlockKind};

    use crate::client::physics::tools::{Tool, ToolKind, ToolMat};

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
