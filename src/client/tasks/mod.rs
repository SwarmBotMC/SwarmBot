/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use std::time::Instant;

use float_ord::FloatOrd;

use bridge::*;
use compound::*;
use delay::*;
use eat::*;
use fall_bucket::*;
use lazy::*;
use lazy_stream::*;
use mine::*;
use mine_column::*;
use mine_goto::*;
use mine_layer::*;
use mine_region::*;
use navigate::*;
use pillar::*;
use pillar_and_mine::*;
use stream::*;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::{PlayerProblem, Problem};
use crate::client::pathfind::implementations::novehicle::{BlockGoalCheck, BlockHeuristic, BlockNearGoalCheck, CenterChunkGoalCheck, ChunkHeuristic, TravelChunkProblem, TravelProblem};
use crate::client::pathfind::moves::CardinalDirection;
use crate::client::pathfind::traits::{GoalCheck, Heuristic};
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::tasks::navigate::{BlockTravelTask, ChunkTravelTask, NavigateProblem};
use crate::client::timing::Increment;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState, SimpleType};
use crate::storage::blocks::ChunkLocation;
use crate::types::{Direction, Displacement, Location};

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
pub mod bridge;
pub mod fall_bucket;

#[enum_dispatch]
pub trait TaskTrait {
    /// return true if done
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool;

    /// Do an expensive part of the task. This is done in a multi-threaded environment. An example of
    /// This has a default implementation of nothing. However, tasks like pathfinding use this. The task
    /// MUST end by the given {end_by} duration else the game loop is held up. This is called every game
    /// loop cycle so if the task hasn't finished it by {end_by} it should instead until this function
    /// is called again.
    fn expensive(&mut self, end_by: Instant, local: &mut LocalState, global: &GlobalState) {}
}

// use eat::EatTask;
// use bridge::BridgeTask;
// use mine_column::MineColumnTask;
// use mine::MineTask;
// use navigate::*;
// use pillar::PillarTask;
// use delay::DelayTask;
// use pillar_and_mine::PillarAndMineTask;
// use mine_layer::MineLayerTask;
// use fall_bucket::FallBucketTask;
// use compound::CompoundTask;

pub type GoMineTopTask = LazyTask<GoMineTop>;
pub type MineRegionTask = LazyStream<MineRegion>;

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(TaskTrait)]
pub enum Task {
    CompoundTask,
    EatTask,
    MineRegionTask,
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
