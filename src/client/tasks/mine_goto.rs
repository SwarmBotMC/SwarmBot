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

use crate::client::{
    state::{
        global::{mine_alloc::MineAlloc, GlobalState},
        local::LocalState,
    },
    tasks::{lazy::Lazy, pillar_and_mine::PillarAndMineTask, Task},
};
use interfaces::types::BlockLocation;

pub struct GoMineTop;

impl Lazy for GoMineTop {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task {
        let BlockLocation { x, y, z } = local.physics.location().into();
        let mut highest_y = y - 1;

        for on_y in y..256 {
            for on_x in (x - MineAlloc::REGION_R)..=(x + MineAlloc::REGION_R) {
                for on_z in (z - MineAlloc::REGION_R)..=(z + MineAlloc::REGION_R) {
                    let location = BlockLocation::new(on_x, on_y, on_z);
                    if let Some(block) = global.blocks.get_block_exact(location) {
                        if block.kind().mineable(&global.block_data) {
                            highest_y = on_y;
                        }
                    }
                }
            }
        }

        // println!("highest y {}", highest_y);

        PillarAndMineTask::pillar_and_mine(highest_y as u32).into()
    }
}
