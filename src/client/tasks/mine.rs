/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:15 AM
 */

use float_ord::FloatOrd;
use itertools::Itertools;

use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState};
use crate::types::{Displacement, Location};
use crate::client::tasks::TaskTrait;

pub struct MineTask {
    pub ticks: usize,
    pub first: bool,
    pub location: BlockLocation,
    pub face: Face,
    pub look_location: Location,
}

impl MineTask {
    pub fn new(location: BlockLocation, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> MineTask {
        let kind = global.blocks.get_block_kind(location).unwrap();
        let tool = local.inventory.switch_tool(out);

        let ticks = tool.wait_time(kind, false, true, &global.block_data) + 1;

        let eye_loc = local.physics.location() + Displacement::EYE_HEIGHT;
        let faces = location.faces();
        let min_position = faces.iter().position_min_by_key(|loc| FloatOrd(loc.dist2(eye_loc))).unwrap();
        let look_location = faces[min_position];
        let face = Face::from(min_position as u8);

        Self {
            ticks,
            location,
            face,
            first: true,
            look_location,
        }
    }
}

impl TaskTrait for MineTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        local.physics.look_at(self.look_location);

        if self.first {
            out.swing_arm();
            self.first = false;
            out.mine(self.location, Mine::Start, Face::PosY);
        }

        out.swing_arm();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, Face::PosY);
            global.blocks.set_block(self.location, BlockState::AIR);
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}
