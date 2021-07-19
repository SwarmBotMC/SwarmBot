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
use crate::client::tasks::compound::CompoundTask;
use crate::client::tasks::delay::DelayTask;
use crate::client::tasks::fall_bucket::FallBucketTask;
use crate::client::tasks::lazy::LazyTask;
use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::tasks::mine_goto::GoMineTop;
use crate::client::tasks::mine_layer::MineLayer;
use crate::client::tasks::stream::TaskStream;
use crate::client::tasks::Task;
use crate::protocol::InterfaceOut;
use crate::storage::block::BlockLocation;

pub struct MineColumn;

impl MineColumn {
    pub const MIN_MINE_LOC: i16 = 11;
}

impl TaskStream for MineColumn {
    fn poll(&mut self, _out: &mut impl InterfaceOut, local: &mut LocalState, _global: &mut GlobalState) -> Option<Task> {
        let mine_loc = BlockLocation::from(local.physics.location()).below();
        if mine_loc.y >= Self::MIN_MINE_LOC {
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
