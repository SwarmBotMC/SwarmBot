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

use crate::client::tasks::stream::TaskStream;
use crate::protocol::{InterfaceOut, Face};
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;
use crate::client::tasks::Task;
use crate::client::tasks::lazy_stream::LazyStream;
use crate::types::Displacement;
use std::collections::HashSet;
use crate::client::tasks::mine::MineTask;
use crate::client::tasks::pillar::PillarTask;


pub type PillarAndMineTask = LazyStream<PillarOrMine>;

impl PillarAndMineTask {
    pub fn pillar_and_mine(height: u32) -> Self {
        let state = PillarOrMine { height };
        Self::from(state)
    }
}

pub struct PillarOrMine {
    height: u32,
}

impl TaskStream for PillarOrMine {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task> {

        let current_height = (local.physics.location().y).floor() as u32;

        // > not >= because we are considering block height
        if current_height > self.height {
            return None;
        }

        let above1 = local.physics.location() + Displacement::new(0., 2.5, 0.);
        let mut set = HashSet::new();
        local.physics.in_cross_section(above1, &global.blocks, &mut set);

        macro_rules! mine_task {
            ($position:expr) => {{
                let mut task = MineTask::new($position, out, local, global);
                task.set_face(Face::NegY);
                Some(task.into())
            }};
        }

        if let Some(&position) = set.iter().next() {
            mine_task!(position)
        } else {
            let above2 = local.physics.location() + Displacement::new(0., 3.5, 0.);
            local.physics.in_cross_section(above2, &global.blocks, &mut set);
            if let Some(&position) = set.iter().next() {
                mine_task!(position)
            } else {
                local.inventory.switch_block(out);
                Some(PillarTask::new(current_height + 1).into())
            }
        }
    }
}
