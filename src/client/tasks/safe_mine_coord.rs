/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */


use crate::client::state::global::GlobalState;
use crate::client::state::global::mine_alloc::MineAlloc;
use crate::client::state::local::LocalState;
use crate::client::tasks::center::CenterTask;
use crate::client::tasks::compound::CompoundTask;
use crate::client::tasks::delay::DelayTask;
use crate::client::tasks::lazy::{Lazy, LazyTask};
use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::tasks::mine_column::MineColumn;
use crate::client::tasks::mine_goto::GoMineTop;
use crate::client::tasks::Task;
use crate::storage::block::{BlockLocation, BlockLocation2D};

pub struct SafeMineRegion;

impl Lazy for SafeMineRegion {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task {
        let location = BlockLocation::from(local.physics.location());
        let center = BlockLocation2D::from(location);

        // if we should skip this region. For example, if there is water or lava we will want to avoid it
        let avoid = MineAlloc::locations_extra(center)
            .any(|loc| {

                // there is often lava under bedrock that we don't really care about
                if loc.y < MineColumn::MIN_MINE_LOC {
                    return false;
                }

                match global.blocks.get_block_exact(loc).map(|x| x.kind().id()) {
                    // water or lava
                    Some(8..=11) => {
                        println!("skipping region {}, {} because of {:?} at {}", center.x, center.z, global.blocks.get_block_exact(loc), loc);
                        true
                    }
                    _ => false
                }
            });

        if avoid {
            DelayTask(0).into()
        } else {
            let mut compound = CompoundTask::default();

            compound.add(CenterTask)
                .add(LazyTask::from(GoMineTop))
                .add(LazyStream::from(MineColumn));

            compound.into()
        }
    }
}
