/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:21 AM
 */

use crate::client::tasks::stream::TaskStream;
use crate::protocol::InterfaceOut;
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

        let current_height = local.physics.location().y as u32;
        if self.height == current_height {
            return None;
        }

        let above = local.physics.location() + Displacement::new(0., 3.5, 0.);
        let mut set = HashSet::new();
        local.physics.in_cross_section(above, &global.blocks, &mut set);
        if let Some(position) = set.into_iter().next() {
            local.inventory.switch_tool(out);
            Some(MineTask::new(position, out, local, global).into())
        } else {
            local.inventory.switch_block(out);
            Some(PillarTask::new(current_height + 1).into())
        }
    }
}
