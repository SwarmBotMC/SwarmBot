/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:21 AM
 */

use crate::client::tasks::TaskTrait;
use crate::protocol::InterfaceOut;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;
use crate::types::Direction;
use crate::storage::block::{SimpleType, BlockLocation};

pub struct PillarTask {
    dest_y: u32,
    jumped: bool,
}

impl PillarTask {
    pub fn new(dest_y: u32) -> PillarTask {
        Self {
            dest_y,
            jumped: false,
        }
    }
}

impl TaskTrait for PillarTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        local.inventory.switch_block(out);

        if local.physics.on_ground() && local.physics.location().y as u32 == self.dest_y {
            return true;
        }

        if local.physics.on_ground() {
            local.physics.jump();
            self.jumped = true;
        }

        let down = Direction {
            yaw: 90.,
            pitch: 90.,
        };

        local.physics.look(down);

        if local.physics.at_apex() && self.jumped {
            let below = BlockLocation::from(local.physics.location()).below();
            let against = below.below();
            if global.blocks.get_block_simple(against) == Some(SimpleType::Solid) {
                local.physics.place_hand(against);
            }
        }

        false
    }
}
