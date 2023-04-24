use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Material {
    Generic,
    Rock,
    Dirt,
    Wood,
    Plant,
    Web,
    Wool,
}

impl Default for Material {
    fn default() -> Self {
        Self::Generic
    }
}

/// Uses prismarine.js block data. We comment out the fields that we do not use
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawBlock {
    pub id: u32,
    // pub display_name: String,
    // pub name: String,
    pub hardness: Option<f64>,
    pub harvest_tools: Option<HashMap<u32, bool>>,
    pub material: Option<Material>,
    // pub stack_size: u32,
    // pub diggable: bool,
    // pub bounding_box: String,
    // drops: [],
    // pub transparent: bool,
    // pub emit_light: u32,
    // pub filter_light: u32,
    // pub resistance: f64
}

/// Uses prismarine.js food data. We comment out the fields that we do not use
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawFood {
    pub id: u32,
}

pub struct Block {
    pub id: u32,
    pub hardness: Option<f64>,
    pub harvest_tools: Vec<u32>,
    pub material: Material,
}

impl From<RawBlock> for Block {
    fn from(block: RawBlock) -> Self {
        Self {
            id: block.id,
            hardness: block.hardness,
            harvest_tools: block
                .harvest_tools
                .unwrap_or_default()
                .into_iter()
                .filter_map(|(k, v)| v.then_some(k))
                .collect(),
            material: block.material.unwrap_or_default(),
        }
    }
}

pub struct BlockData {
    // lookup by id
    block_lookup: HashMap<u32, Block>,
    food_lookup: HashSet<u32>,
}

impl Default for BlockData {
    fn default() -> Self {
        Self::read().unwrap()
    }
}

impl BlockData {
    pub fn by_id(&self, id: u32) -> Option<&Block> {
        self.block_lookup.get(&id)
    }

    pub fn is_food(&self, id: u32) -> bool {
        self.food_lookup.contains(&id)
    }

    pub fn read() -> Result<BlockData, serde_json::Error> {
        let blocks: Vec<RawBlock> = {
            let s = include_str!("blocks.json");
            serde_json::from_str(s)?
        };

        let foods: Vec<RawFood> = {
            let s = include_str!("foods.json");
            serde_json::from_str(s)?
        };

        let food_lookup: HashSet<_> = foods.into_iter().map(|food| food.id).collect();

        let blocks = blocks.into_iter().map(Block::from);

        let block_lookup = blocks.map(|elem| (elem.id, elem)).collect();

        Ok(BlockData {
            block_lookup,
            food_lookup,
        })
    }
}
