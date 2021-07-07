/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/6/21, 11:47 PM
 */


use std::collections::VecDeque;

use rayon::prelude::ParallelSliceMut;

use crate::storage::block::BlockLocation2D;

/// Represents the bottom left corner of a region
#[derive(Debug)]
struct MineRegion(BlockLocation2D);

/// Allocates mine regions to bots
#[derive(Debug, Default)]
pub struct MineAlloc {
    regions: VecDeque<MineRegion>,
}

pub enum MinePreference {
    FromDist,
    ToDist,
}

impl MineAlloc {
    pub const REGION_R: i32 = 3;
    pub const REGION_WIDTH: i32 = Self::REGION_R * 2 + 1;

    pub fn cancel(&mut self) {
        self.regions.clear();
    }

    pub fn obtain_region(&mut self) -> Option<BlockLocation2D> {
        let BlockLocation2D { x, z } = self.regions.pop_front()?.0;
        let centered = BlockLocation2D::new(x + Self::REGION_WIDTH / 2, z + Self::REGION_WIDTH / 2);
        Some(centered)
    }

    pub fn mine(&mut self, from: BlockLocation2D, to: BlockLocation2D, preference: Option<MinePreference>) {

        // we must complete previous operation before we continue.
        // this also is important because right now every time a bot reads #mine in chat it executes this
        // and if we had 100 bots that would mean this would execute 100 times = bad
        if !self.regions.is_empty() {
            return;
        }

        let mut vec = Vec::new();

        for x in (from.x..=to.x).step_by(Self::REGION_WIDTH as usize) {
            for z in (from.z..=to.z).step_by(Self::REGION_WIDTH as usize) {
                let loc = BlockLocation2D::new(x, z);
                vec.push(MineRegion(loc));
            }
        }

        if let Some(preference) = preference {
            match preference {
                MinePreference::FromDist => {
                    vec.par_sort_unstable_by_key(|region| {
                        region.0.dist2(from)
                    })
                }
                MinePreference::ToDist => {
                    vec.par_sort_unstable_by_key(|region| {
                        region.0.dist2(to)
                    })
                }
            }
        }

        for elem in vec {
            self.regions.push_back(elem);
        }
    }
}
