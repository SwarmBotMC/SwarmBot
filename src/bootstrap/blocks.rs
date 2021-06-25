use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::collections::HashMap;

/// Uses prismarine.js block data. We comment out the fields that we do not use
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: u32,
    // pub display_name: String,
    // pub name: String,
    pub hardness: Option<f64>,
    // pub stack_size: u32,
    // pub diggable: bool,
    // pub bounding_box: String,
    // drops: [],
    // pub transparent: bool,
    // pub emit_light: u32,
    // pub filter_light: u32,
    // pub resistance: f64
}

pub struct BlockData {
    // lookup by id
    lookup: HashMap<u32, Block>
}

impl BlockData {

    pub fn by_id(&self, id: u32) -> Option<&Block> {
        self.lookup.get(&id)
    }

    pub fn read() -> Result<BlockData, serde_json::Error> {
        let reader = OpenOptions::new().read(true).open("blocks.json").unwrap();

        let blocks: Vec<Block> = serde_json::from_reader(reader)?;

        let lookup = blocks.into_iter()
            .map(|elem| (elem.id, elem))
            .collect();

        Ok(BlockData {
            lookup
        })
    }
}
