/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:46 AM
 */


use crate::client::state::global::GlobalState;
use crate::client::state::global::mine_alloc::MineAlloc;
use crate::client::state::local::LocalState;
use crate::client::tasks::lazy::Lazy;
use crate::client::tasks::Task;
use crate::storage::block::{BlockLocation, SimpleType};
use crate::client::tasks::pillar_and_mine::{PillarAndMineTask};

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

        println!("highest y {}", highest_y);

        PillarAndMineTask::pillar_and_mine(highest_y as u32).into()
    }
}
