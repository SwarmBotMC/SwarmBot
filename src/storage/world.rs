use std::collections::HashMap;

use crate::storage::block::{BlockApprox, BlockLocation, SimpleType};
use crate::storage::chunk::ChunkColumn;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ChunkLocation(pub i32, pub i32);

#[derive(Default)]
pub struct WorldBlocks {
    storage: HashMap<ChunkLocation, ChunkColumn>,
}

impl WorldBlocks {

    pub fn add_column(&mut self, location: ChunkLocation, column: ChunkColumn){
        self.storage.insert(location, column);
    }
    pub fn get_block(&self, location: BlockLocation) -> Option<BlockApprox> {
        let BlockLocation(x, y, z) = location;

        let y = y as u8;

        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        let x = (x - (chunk_x << 4)) as u8;
        let z = (z - (chunk_z << 4)) as u8;

        let chunk_x = chunk_x as i32;
        let chunk_z = chunk_z as i32;

        let loc = ChunkLocation(chunk_x, chunk_z);
        let chunk = self.storage.get(&loc)?;
        let block = chunk.get_block(x, y, z);
        Some(block)
    }

    pub fn get_block_simple(&self, location: BlockLocation) -> Option<SimpleType> {
        self.get_block(location).map(|x| x.s_type())
    }
}
