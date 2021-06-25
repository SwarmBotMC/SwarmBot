use std::collections::HashMap;
use std::convert::TryFrom;

use crate::storage::block::{BlockApprox, BlockLocation, BlockState, SimpleType};
use crate::storage::chunk::ChunkColumn;
use std::num::TryFromIntError;
use itertools::Itertools;

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

        let y = match u8::try_from(y) {
            Ok(inner) => inner,
            Err(_) => {
                panic!("y {} is not in range of u8. This is not yet supported", y)
            }
        };

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

    pub fn select(&'a self, around: BlockLocation, selector: impl FnMut(BlockState) -> bool + 'a + Copy) -> impl Iterator<Item=BlockLocation> + 'a {
        self.storage.iter()
            .sorted_unstable_by_key(|(loc,_)|{
                let chunk_center_loc = BlockLocation::new(loc.0 << 4, 64, loc.1 << 4);
                chunk_center_loc.dist2(around)
            })
            .filter_map(|(loc, column)| {
                match column {
                    ChunkColumn::HighMemory { data } => {
                        Some((loc, data))
                    }
                    _ => { None }
                }
            })
            .flat_map(move |(loc, column)| {
                let start_x = loc.0 << 4;
                let start_z = loc.1 << 4;
                column.select(selector).map(move |idx|{
                    let x = idx % 16;
                    let leftover = idx >> 4;
                    let z = leftover % 16;
                    let y = leftover / 16;
                    BlockLocation::new(x as i32 + start_x, y as i16, z as i32 + start_z)
                })
            })
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
            None => {},
            Some(column) => column.set_block(x, y, z, block)
        };
    }

    pub fn get_block_simple(&self, location: BlockLocation) -> Option<SimpleType> {
        if location.y < 0 {
            return None;
        }
        self.get_block(location).map(|x| x.s_type())
    }
}
