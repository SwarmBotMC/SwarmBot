/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::collections::HashMap;

use rand::{Rng, SeedableRng};
use rand::rngs::{StdRng, ThreadRng};

use crate::schematic::Schematic;
use crate::storage::block::{BlockApprox, BlockLocation, BlockState, SimpleType, BlockKind};
use crate::storage::chunk::ChunkColumn;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ChunkLocation(pub i32, pub i32);

#[derive(Default)]
pub struct WorldBlocks {
    storage: HashMap<ChunkLocation, ChunkColumn>,
}

impl WorldBlocks {
    /// A world that is flat at y=0 in a 100 block radius from 0,0
    pub fn flat() -> WorldBlocks {
        let mut world = WorldBlocks::default();
        for x in -100..=100 {
            for z in -100..=100 {
                let loc = BlockLocation::new(x, 0, z);
                world.set_block(loc, BlockState::STONE);
            }
        }
        world
    }

    /// similar to a bedrock floor. (0,0) is guaranteed to be a solid as well as (950, 950)
    pub fn set_random_floor(&mut self) {
        const RADIUS: i32 = 1000;
        let mut rdm = StdRng::seed_from_u64(12338971);
        for x in -RADIUS..=RADIUS {
            for z in -RADIUS..=RADIUS {
                let res = rdm.gen_range(0..5);
                let loc = BlockLocation::new(x, 0, z);
                if res == 0 {
                    self.set_block(loc, BlockState::STONE);
                }
            }
        }

        self.set_block(BlockLocation::default(), BlockState::STONE);
        self.set_block(BlockLocation::new(950, 0, 950), BlockState::STONE);
    }

    pub fn load(&mut self, schematic: &Schematic) {
        for (location, state) in schematic.blocks() {
            self.set_block(location, state)
        }
    }

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

        let column = self.storage.entry(loc).or_default();
        column.set_block(x, y, z, block);
    }

    pub fn get_block_simple(&self, location: BlockLocation) -> Option<SimpleType> {
        let block = self.get_block(location)?;
        Some(block.s_type())
    }

    pub fn get_block_exact(&self, location: BlockLocation) -> Option<BlockState> {
        let block = self.get_block(location)?;
        match block {
            BlockApprox::Realized(state) => Some(state),
            _ => None
        }
    }

    pub fn get_block_kind(&self, location: BlockLocation) -> Option<BlockKind> {
        let block =self.get_block_exact(location)?;
        Some(block.kind())
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::block::{BlockApprox, BlockLocation, BlockState};
    use crate::storage::blocks::WorldBlocks;

    #[test]
    fn test_get_set() {
        let mut world = WorldBlocks::default();

        let loc = BlockLocation::new(0, 0, 0);


        {
            world.set_block(loc, BlockState::STONE);
            let got = world.get_block(loc);
            assert_matches!(got , Some(BlockApprox::Realized(BlockState::STONE)));
        }

        {
            let up = loc + BlockLocation::new(0, 1, 0);
            let given = BlockState(123);
            world.set_block(up, given);

            let got_up = world.get_block(up);
            assert_matches!(got_up , Some(BlockApprox::Realized(given)));

            let got = world.get_block(loc);
            assert_matches!(got , Some(BlockApprox::Realized(BlockState::STONE)));
        }

        {
            world.set_block(loc, BlockState::AIR);
            let got = world.get_block(loc);
            assert_matches!(got , Some(BlockApprox::Realized(BlockState::AIR)));
        }
    }
}
