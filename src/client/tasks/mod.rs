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
use std::time::Instant;



use bridge::*;
use compound::*;
use delay::*;
use eat::*;
use fall_bucket::*;
use lazy::*;

use mine::*;
use mine_column::*;
use mine_goto::*;
use mine_layer::*;
use mine_region::*;
use navigate::*;
use pillar::*;
use pillar_and_mine::*;










use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::tasks::navigate::{BlockTravelTask, ChunkTravelTask};

use crate::protocol::{InterfaceOut};




pub mod delay;
pub mod compound;
pub mod eat;
pub mod navigate;
pub mod mine;
pub mod lazy_stream;
pub mod stream;
pub mod lazy;
pub mod mine_layer;
pub mod mine_column;
pub mod pillar;
pub mod pillar_and_mine;
pub mod mine_goto;
pub mod mine_region;
pub mod safe_mine_coord;
pub mod bridge;
pub mod fall_bucket;
pub mod center;

use center::CenterTask;
use crate::client::tasks::safe_mine_coord::SafeMineRegion;

#[enum_dispatch]
pub trait TaskTrait {
    /// return true if done
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool;

    /// Do an expensive part of the task. This is done in a multi-threaded environment. An example of
    /// This has a default implementation of nothing. However, tasks like pathfinding use this. The task
    /// MUST end by the given {end_by} duration else the game loop is held up. This is called every game
    /// loop cycle so if the task hasn't finished it by {end_by} it should instead until this function
    /// is called again.
    fn expensive(&mut self, _end_by: Instant, _local: &mut LocalState, _global: &GlobalState) {}
}

pub type GoMineTopTask = LazyTask<GoMineTop>;
pub type MineRegionTask = LazyStream<MineRegion>;
pub type SafeMineRegionTask = LazyTask<SafeMineRegion>;

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(TaskTrait)]
pub enum Task {
    CompoundTask,
    EatTask,
    MineRegionTask,
    SafeMineRegionTask,
    CenterTask,
    BridgeTask,
    GoMineTopTask,
    MineColumnTask,
    MineTask,
    BlockTravelNearTask,
    BlockTravelTask,
    ChunkTravelTask,
    PillarTask,
    DelayTask,
    PillarAndMineTask,
    MineLayerTask,
    FallBucketTask,
}
