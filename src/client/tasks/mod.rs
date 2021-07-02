/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use std::collections::VecDeque;
use std::time::Instant;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::{MoveNode, MoveRecord};
use crate::client::pathfind::implementations::{PlayerProblem, Problem};
use crate::client::pathfind::implementations::novehicle::{BlockGoalCheck, BlockHeuristic, ChunkGoalCheck, ChunkHeuristic, TravelChunkProblem, TravelProblem};
use crate::client::pathfind::incremental::PathResult;
use crate::client::pathfind::traits::{GoalCheck, Heuristic};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::blocks::ChunkLocation;

#[enum_dispatch]
pub trait TaskTrait {
    /// return true if done
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool;
    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {}
}

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(TaskTrait)]
pub enum Task {
    EatTask,
    MineTask,
    PillarTask,
    BlockTravelTask,
    ChunkTravelTask,
    FallBucketTask,
    CompoundTask,
}

pub struct CompoundTask {
    tasks: VecDeque<Task>,
}

pub struct NavigateProblem<H: Heuristic, G: GoalCheck> {
    calculate: bool,
    problem: PlayerProblem<H, G>,
    follower: Option<Follower>,
}

impl<H: Heuristic, G: GoalCheck> NavigateProblem<H, G> {
    fn raw(problem: PlayerProblem<H, G>) -> NavigateProblem<H, G> {
        Self {
            calculate: true,
            problem,
            follower: None,
        }
    }
}


impl<H: Heuristic + Send + Sync, G: GoalCheck + Send+ Sync> TaskTrait for NavigateProblem<H, G> {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let follower = match self.follower.as_mut() {
            None => return false,
            Some(inner) => inner
        };

        if follower.should_recalc() {
            self.problem.recalc(MoveNode::simple(local.physics.location().into()));
            self.calculate = true;
        }

        match follower.follow(local, global) {
            FollowResult::Failed => {
                self.follower = None;
                self.problem.recalc(MoveNode::simple(local.physics.location().into()));
                self.calculate = true;
                false
            }
            FollowResult::InProgress => false,
            FollowResult::Finished => true
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

pub type ChunkTravelTask = NavigateProblem<ChunkHeuristic, ChunkGoalCheck>;
pub type BlockTravelTask = NavigateProblem<BlockHeuristic, BlockGoalCheck>;

impl ChunkTravelTask {
    pub fn new(goal: ChunkLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_chunk(start, goal);
        Self::raw(problem)
    }
}

impl BlockTravelTask {
    pub fn new(goal: BlockLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_block(start, goal);
        Self::raw(problem)
    }
}

impl CompoundTask {
    pub fn new<const T: usize>(tasks: [Task; T]) -> CompoundTask {
        CompoundTask {
            tasks: IntoIterator::into_iter(tasks).collect()
        }
    }
}

impl TaskTrait for CompoundTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let front = match self.tasks.front_mut() {
            None => return true,
            Some(inner) => inner
        };

        let finished = front.tick(out, local, global);

        if finished {
            println!("next");
            self.tasks.pop_front();
        }

        self.tasks.is_empty()
    }
}

pub struct EatTask {
    pub ticks: usize,
}

impl TaskTrait for EatTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, _: &mut LocalState, _: &mut GlobalState) -> bool {
        if self.ticks == 0 {
            out.finish_eating();
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}

pub struct MineTask {
    pub ticks: usize,
    pub location: BlockLocation,
    pub face: Face,
}

impl TaskTrait for MineTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, _: &mut LocalState, global: &mut GlobalState) -> bool {
        out.left_click();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, self.face);
            global.world_blocks.set_block(self.location, BlockState::AIR);
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}


pub struct PillarTask {
    count: u32,
    place_loc: BlockLocation,
}

impl PillarTask {
    pub fn new(count: u32, local: &LocalState) -> PillarTask {
        Self {
            count,
            place_loc: BlockLocation::from(local.physics.location()).below()
        }
    }
}

impl TaskTrait for PillarTask {
    fn tick(&mut self, _: &mut impl InterfaceOut, local: &mut LocalState, _: &mut GlobalState) -> bool {
        local.physics.jump();

        if local.physics.at_apex() {
            local.physics.place_hand(self.place_loc);
            self.count -= 1;
            self.place_loc = self.place_loc.above()
        }

        self.count == 0
    }
}



#[derive(Default)]
pub struct FallBucketTask {
    placed: bool,
    ticks_since_place: u32,
    location: Option<BlockLocation>,
}

impl TaskTrait for FallBucketTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        if self.placed {
            self.ticks_since_place += 1;
            if self.ticks_since_place > 10 {
                out.right_click();
                out.place_block(self.location.unwrap(), Face::PosY);
                return true;
            }
            return false;
        }

        let current_loc = local.physics.location();
        let below = global.world_blocks.first_below(current_loc.into());
        match below {
            None => {}
            Some((location, _)) => {
                local.physics.look_at(location.center_bottom());
                let dy = current_loc.y - (location.y as f64 + 1.0);
                if dy < 3.4 {
                    out.right_click();
                    out.place_block(location, Face::PosY);
                    self.location = Some(location);
                    self.placed = true;
                    self.ticks_since_place = 0;
                }
            }
        }

        false
    }
}
