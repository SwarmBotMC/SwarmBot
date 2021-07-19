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
use crate::client::state::local::LocalState;
use crate::client::tasks::TaskTrait;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState};

pub struct MineTask {
    ticks: usize,
    first: bool,
    location: BlockLocation,
    face: Face,
}

impl MineTask {
    pub fn new(location: BlockLocation, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> MineTask {
        let kind = global.blocks.get_block_kind(location).unwrap();

        let tool = local.inventory.switch_tool(kind, &global.block_data, out);

        let ticks = tool.wait_time(kind, false, true, &global.block_data) + 1;

        Self {
            ticks,
            location,
            face: Face::PosY,
            first: true,
        }
    }

    pub fn set_face(&mut self, face: Face) {
        self.face = face;
    }
}

impl TaskTrait for MineTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let look_loc = self.location.faces()[self.face as usize];
        local.physics.look_at(look_loc);

        if self.first {
            out.swing_arm();
            self.first = false;
            out.mine(self.location, Mine::Start, self.face);
        }

        // println!("mining {}", self.location);

        out.swing_arm();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, self.face);
            global.blocks.set_block(self.location, BlockState::AIR);
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}
