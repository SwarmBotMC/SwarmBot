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

pub struct MineColumn;

impl TaskStream for MineColumn {
    fn poll(&mut self, _out: &mut impl InterfaceOut, local: &mut LocalState, _global: &mut GlobalState) -> Option<Task> {
        let mine_loc = BlockLocation::from(local.physics.location()).below();
        if mine_loc.y >= 5 {
            let mut compound = CompoundTask::default();

            compound
                .add(LazyStream::from(MineLayer))
                .add(DelayTask(5))
                .add(FallBucketTask::default())
                .add(LazyTask::from(GoMineTop));

            Some(compound.into())
        } else {
            None
        }
    }
}

pub type MineColumnTask = LazyStream<MineColumn>;
