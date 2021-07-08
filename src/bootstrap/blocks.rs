/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::collections::HashMap;
use std::fs::OpenOptions;

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

pub struct Block {
    pub id: u32,
    pub hardness: Option<f64>,
    pub harvest_tools: Vec<u32>,
    pub material: Material,
}

impl From<RawBlock> for Block {
    fn from(raw: RawBlock) -> Self {
        Self {
            id: raw.id,
            hardness: raw.hardness,
            harvest_tools: raw.harvest_tools.unwrap_or_default().into_iter()
                .filter_map(|(k,v)| v.then(||k)).collect(),
            material: raw.material.unwrap_or_default()
        }
    }
}

pub struct BlockData {
    // lookup by id
    lookup: HashMap<u32, Block>,
}

impl Default for BlockData {
    fn default() -> Self {
        Self::read().unwrap()
    }
}

impl BlockData {
    pub fn by_id(&self, id: u32) -> Option<&Block> {
        self.lookup.get(&id)
    }

    pub fn read() -> Result<BlockData, serde_json::Error> {
        let reader = OpenOptions::new().read(true).open("blocks.json").unwrap();

        let blocks: Vec<RawBlock> = serde_json::from_reader(reader)?;

        let blocks = blocks.into_iter().map(Block::from);

        let lookup = blocks
            .map(|elem| (elem.id, elem))
            .collect();

        Ok(BlockData {
            lookup
        })
    }
}
