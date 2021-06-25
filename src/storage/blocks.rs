use std::collections::HashMap;

use crate::storage::block::{BlockApprox, BlockLocation, BlockState, SimpleType};
use crate::storage::chunk::ChunkColumn;
use std::convert::TryFrom;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ChunkLocation(pub i32, pub i32);

#[derive(Default)]
pub struct WorldBlocks {
    storage: HashMap<ChunkLocation, ChunkColumn>,
}

impl WorldBlocks {
    pub fn add_column(&mut self, location: ChunkLocation, column: ChunkColumn) {
        self.storage.insert(location, column);
    }

    pub fn get_block(&self, location: BlockLocation) -> Option<BlockApprox> {
        let BlockLocation { x, y, z } = location;

        let y = u8::try_from(y).expect("y not in the range of u8 is not yet supported");

        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        let x = (x - (chunk_x << 4)) as u8;
        let z = (z - (chunk_z << 4)) as u8;

        let chunk_x = chunk_x as i32;
        let chunk_z = chunk_z as i32;

        let loc = ChunkLocation(chunk_x, chunk_z);
        let column = self.storage.get(&loc)?;
        let block = column.get_block(x, y as u8, z);
        Some(block)
    }

    pub fn set_block(&mut self, location: BlockLocation, block: BlockState) {
        let BlockLocation { x, y, z } = location;

        let y = y as u8;

        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        let x = (x - (chunk_x << 4)) as u8;
        let z = (z - (chunk_z << 4)) as u8;

        let chunk_x = chunk_x as i32;
        let chunk_z = chunk_z as i32;


        let loc = ChunkLocation(chunk_x, chunk_z);

        match self.storage.get_mut(&loc) {
            None => return,
            Some(column) => column.set_block(x, y, z, block)
        };
    }

    pub fn get_block_simple(&self, location: BlockLocation) -> Option<SimpleType> {
        self.get_block(location).map(|x| x.s_type())
    }
}
