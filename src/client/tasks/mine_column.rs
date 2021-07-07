/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:19 AM
 */

use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::state::local::LocalState;
use crate::storage::block::BlockLocation;
use crate::client::tasks::stream::TaskStream;
use crate::protocol::InterfaceOut;
use crate::client::state::global::GlobalState;
use crate::client::tasks::fall_bucket::FallBucketTask;

use crate::client::tasks::delay::DelayTask;
use crate::client::tasks::compound::CompoundTask;
use crate::client::tasks::mine_layer::MineLayer;
use crate::client::tasks::Task;
use crate::client::tasks::mine_goto::GoMineTop;
use crate::client::tasks::lazy::LazyTask;

#[derive(Default)]
pub struct MineColumn {
    on_loc: Option<BlockLocation>,
}

impl TaskStream for MineColumn {
    fn poll(&mut self, _out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task> {
        let on_loc = self.on_loc.get_or_insert_with(||BlockLocation::from(local.physics.location()).below());
        if on_loc.y > 5 {
            let mine_task = MineLayer::new(*on_loc, global)?;
            let task = LazyStream::from(mine_task);
            let mut compound = CompoundTask::default();

            compound
                .add(task)
                .add(DelayTask(5))
                .add(FallBucketTask::default())
                .add(LazyTask::from(GoMineTop));

            on_loc.y -= 1;

            Some(compound.into())
        } else {
            None
        }
    }
}

pub type MineColumnTask = LazyStream<MineColumn>;
