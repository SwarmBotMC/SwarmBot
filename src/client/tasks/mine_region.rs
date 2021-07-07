/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:30 AM
 */


use crate::client::pathfind::implementations::novehicle::TravelProblem;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::compound::CompoundTask;
use crate::client::tasks::lazy::LazyTask;
use crate::client::tasks::mine_column::{MineColumn};
use crate::client::tasks::mine_goto::GoMineTop;
use crate::client::tasks::navigate::{NavigateProblem};
use crate::client::tasks::stream::TaskStream;
use crate::client::tasks::Task;
use crate::protocol::InterfaceOut;

use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::tasks::center::CenterTask;

pub struct MineRegion;

impl TaskStream for MineRegion {
    fn poll(&mut self, _out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task> {
        let goal = global.mine.obtain_region()?;
        let start = local.physics.location();

        let mut compound = CompoundTask::default();
        let problem = TravelProblem::navigate_near_block(start.into(), goal, 0.0, false);
        let nav = NavigateProblem::from(problem);
        compound.add(nav)
            .add(CenterTask)
            .add(LazyTask::from(GoMineTop))
            .add(LazyStream::from(MineColumn::default()));

        Some(compound.into())
    }
}
