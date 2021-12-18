// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::VecDeque;

use interfaces::types::{BlockLocation, BlockLocation2D};
use itertools::Itertools;
use rayon::prelude::ParallelSliceMut;

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

pub type Locations = impl Iterator<Item = BlockLocation>;

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

    fn locations_rad(center: BlockLocation2D, rad: i32) -> Locations {
        (0..256)
            .cartesian_product(-rad..=rad)
            .cartesian_product(-rad..=rad)
            .map(move |((y, z), x)| BlockLocation::new(center.x + x, y as i16, center.z + z))
    }

    pub fn locations(center: BlockLocation2D) -> Locations {
        Self::locations_rad(center, Self::REGION_R)
    }

    // locations plus 1 block extra
    pub fn locations_extra(center: BlockLocation2D) -> Locations {
        Self::locations_rad(center, Self::REGION_R + 1)
    }

    pub fn mine(
        &mut self,
        from: BlockLocation2D,
        to: BlockLocation2D,
        preference: Option<MinePreference>,
    ) {
        // we must complete previous operation before we continue.
        // this also is important because right now every time a bot reads #mine in chat
        // it executes this and if we had 100 bots that would mean this would
        // execute 100 times = bad
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
                    vec.par_sort_unstable_by_key(|region| region.0.dist2(from))
                }
                MinePreference::ToDist => vec.par_sort_unstable_by_key(|region| region.0.dist2(to)),
            }
        }

        for elem in vec {
            self.regions.push_back(elem);
        }
    }
}
