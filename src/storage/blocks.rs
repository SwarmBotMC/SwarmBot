/*
 * Copyright (c) 2021 Minecraft IGN RevolutionNow - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by RevolutionNow <Xy8I7.Kn1RzH0@gmail.com>, 6/29/21, 8:16 PM
 */

use std::collections::{HashMap};



use crate::storage::block::{BlockApprox, BlockLocation, BlockState, SimpleType};
use crate::storage::chunk::ChunkColumn;

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

    pub fn modify_column(&mut self, location: ChunkLocation, column: ChunkColumn) {
        self.storage.get_mut(&location).unwrap().modify(column);
    }

    pub fn get_block(&self, location: BlockLocation) -> Option<BlockApprox> {
        let BlockLocation { x, y, z } = location;

        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        let x = (x - (chunk_x << 4)) as u8;
        let z = (z - (chunk_z << 4)) as u8;

        let chunk_x = chunk_x as i32;
        let chunk_z = chunk_z as i32;

        let loc = ChunkLocation(chunk_x, chunk_z);
        let column = self.storage.get(&loc)?;

        // this *should* be either the void or the sky (at least pre-1.17)
        // we do this check here because we want to return None if there is no chunk column in that position
        if !(0..256).contains(&y) {
            return Some(BlockApprox::Realized(BlockState::AIR));
        }

        let block = column.get_block(x, y as u8, z);
        Some(block)
    }

    pub fn closest(&'a self, origin: BlockLocation, selector: impl FnMut(BlockState) -> bool + 'a + Copy) -> Option<BlockLocation> {
        self.select(origin, usize::MAX, selector)
            .min_by_key(|loc| loc.dist2(origin))
    }

    pub fn select(&'a self, _around: BlockLocation, _max_chunks: usize, selector: impl FnMut(BlockState) -> bool + 'a + Copy) -> impl Iterator<Item=BlockLocation> + 'a {
        self.storage.iter()
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
                column.select(selector).map(move |idx| {
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
            None => {}
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
