/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:10 AM
 */

use crate::client::pathfind::traits::{Heuristic, GoalCheck};
use crate::client::pathfind::implementations::{PlayerProblem, Problem};
use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::implementations::novehicle::{ChunkHeuristic, CenterChunkGoalCheck, BlockGoalCheck, BlockHeuristic, BlockNearGoalCheck, TravelProblem};
use crate::client::tasks::TaskTrait;
use crate::client::state::local::LocalState;
use crate::protocol::InterfaceOut;
use crate::client::state::global::GlobalState;
use crate::client::pathfind::context::MoveNode;
use std::time::Instant;
use crate::client::timing::Increment;
use crate::storage::blocks::ChunkLocation;
use crate::storage::block::BlockLocation;

pub type ChunkTravelTask = NavigateProblem<ChunkHeuristic, CenterChunkGoalCheck>;
pub type BlockTravelTask = NavigateProblem<BlockHeuristic, BlockGoalCheck>;
pub type BlockTravelNearTask = NavigateProblem<BlockHeuristic, BlockNearGoalCheck>;

impl ChunkTravelTask {
    pub fn new(goal: ChunkLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_center_chunk(start, goal);
        problem.into()
    }
}

impl BlockTravelTask {
    pub fn new(goal: BlockLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_block(start, goal);
        problem.into()
    }
}

pub struct NavigateProblem<H: Heuristic, G: GoalCheck> {
    calculate: bool,
    problem: Box<PlayerProblem<H, G>>,
    follower: Option<Follower>,
}

impl<H: Heuristic, G: GoalCheck> From<PlayerProblem<H, G>> for NavigateProblem<H, G> {
    fn from(problem: PlayerProblem<H, G>) -> Self {
        Self {
            calculate: true,
            problem: box problem,
            follower: None,
        }
    }
}

impl<H: Heuristic + Send + Sync, G: GoalCheck + Send + Sync> TaskTrait for NavigateProblem<H, G> {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let follower = match self.follower.as_mut() {
            None => return false,
            Some(inner) => inner
        };

        if follower.should_recalc() {
            println!("recalc");
            self.problem.recalc(MoveNode::simple(local.physics.location().into()));
            self.calculate = true;
        }

        match follower.follow(local, global) {
            FollowResult::Failed => {
                println!("failed");
                self.follower = None;
                self.problem.recalc(MoveNode::simple(local.physics.location().into()));
                self.calculate = true;
                false
            }
            FollowResult::InProgress => false,
            FollowResult::Finished => {
                println!("finished!");
                true
            }
        }
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        if !self.calculate {
            return;
        }

        let res = self.problem.iterate_until(end_at, local, global);
        match res {
            Increment::Finished(res) => {
                self.calculate = false;
                match self.follower.as_mut() {
                    None => self.follower = Follower::new(res),
                    Some(before) => before.merge(res)
                };
            }

            // Nothing as we are still in progress
            Increment::InProgress => {}
        }
    }
}
