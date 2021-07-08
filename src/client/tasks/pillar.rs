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
use crate::types::{Direction, Displacement};
use crate::storage::block::{SimpleType, BlockLocation};

pub struct PillarTask {
    dest_y: u32,
    jumped: bool,
}

impl PillarTask {
    pub fn new(dest_y: u32) -> PillarTask {
        println!("pillar dest {}", dest_y);
        Self {
            dest_y,
            jumped: false,
        }
    }
}

impl TaskTrait for PillarTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        local.inventory.switch_block(out);

        // equal OR GREATER because we don't want to pillar if we are higher than we need to be
        if local.physics.location().y as u32 >= self.dest_y {
            let below_loc = BlockLocation::from(local.physics.location()).below();

            // return true if block below us is solid
            if global.blocks.get_block_simple(below_loc) == Some(SimpleType::Solid) {
                return true;
            }
        }

        local.physics.jump();

        local.physics.look(Direction::DOWN);


        // subtract a little so we can be conservative with placements
        let location = local.physics.location() - Displacement::new(0., 0.1, 0.);

        let below_block = BlockLocation::from(location).below();
        let two_below = below_block.below();

        let below_valid = match global.blocks.get_block_simple(below_block) {
            Some(SimpleType::Water) | Some(SimpleType::WalkThrough) => true,
            _ => false
        };

        let two_below_valid = match global.blocks.get_block_simple(two_below) {
            Some(SimpleType::Solid) => true,
            _ => false
        };

        if below_valid && two_below_valid {
            let below = BlockLocation::from(local.physics.location()).below();
            let against = below.below();
            if global.blocks.get_block_simple(against) == Some(SimpleType::Solid) {
                local.physics.place_hand(against);
            }
        }

        false
    }
}
