/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use std::collections::VecDeque;
use std::time::Instant;

use float_ord::FloatOrd;
use itertools::Itertools;

use crate::bootstrap::blocks::BlockData;
use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::{MoveNode, MoveRecord};
use crate::client::pathfind::implementations::{PlayerProblem, Problem};
use crate::client::pathfind::implementations::novehicle::{BlockGoalCheck, BlockHeuristic, ChunkGoalCheck, ChunkHeuristic, TravelChunkProblem, TravelProblem};
use crate::client::pathfind::incremental::PathResult;
use crate::client::pathfind::traits::{GoalCheck, Heuristic};
use crate::client::physics::tools::{Material, Tool};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::blocks::ChunkLocation;
use crate::types::{Displacement, Location, Direction};
use crate::client::pathfind::moves::{CardinalDirection, Change};
use crate::client::physics::speed::Speed;
use crate::client::physics::Line;

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

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(TaskTrait)]
pub enum Task {
    EatTask,
    BridgeTask,
    MineTask,
    PillarTask,
    DelayTask,
    LazyTask,
    BlockTravelTask,
    ChunkTravelTask,
    FallBucketTask,
    CompoundTask,
}


pub struct DelayTask(u32);

impl DelayTask {
    pub fn new(ticks: u32) -> Self {
        Self(ticks)
    }
}

impl TaskTrait for DelayTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let ticks_left = self.0;
        if ticks_left == 0 {
            true
        } else {
            self.0 -= 1;
            false
        }
    }
}

#[derive(Default)]
pub struct CompoundTask {
    tasks: VecDeque<Task>,
}


impl CompoundTask {
    pub fn mine_all<T: IntoIterator<Item=BlockLocation>>(blocks: T, local: &LocalState, global: &GlobalState) -> CompoundTask {
        let eye_location = local.physics.location() + Displacement::EYE_HEIGHT;
        let tasks = blocks
            .into_iter()
            .sorted_unstable_by_key(|&loc| FloatOrd(loc.true_center().dist2(eye_location)))
            .map(|loc| MineTask::new(loc, local, global).into())
            .collect();

        Self {
            tasks
        }
    }
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


impl<H: Heuristic + Send + Sync, G: GoalCheck + Send + Sync> TaskTrait for NavigateProblem<H, G> {
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
    pub fn add<T: Into<Task>>(&mut self, task: T) -> &mut Self {
        self.tasks.push_back(task.into());
        self
    }

    pub fn add_lazy<A: Into<Task>, T: FnOnce(&mut LocalState, &GlobalState) -> A + 'static + Sync + Send>(&mut self, block: T) -> &mut Self {
        let lazy = LazyTask::new(|local, global| {
            block(local, global).into()
        });

        self.add(lazy);

        self
    }
}

impl TaskTrait for CompoundTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {

        // this is so we can do multiple 0-tick tasks in a gametick
        while let Some(front) = self.tasks.front_mut() {
            let finished = front.tick(out, local, global);

            if finished {
                self.tasks.pop_front();
            } else {
                return false;
            }
        }

        return true;
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        match self.tasks.front_mut() {
            None => return,
            Some(res) => res.expensive(end_at, local, global)
        };
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
    pub first: bool,
    pub location: BlockLocation,
    pub face: Face,
    pub look_location: Location,
}

impl MineTask {
    pub fn new(location: BlockLocation, local: &LocalState, global: &GlobalState) -> MineTask {
        let kind = global.world_blocks.get_block_kind(location).unwrap();
        let tool = Tool::new(Material::DIAMOND);

        // taking one tick off because most servers are ok with this
        let ticks = tool.wait_time(kind, false, true, 5, &global.block_data) - 1;

        let eye_loc = local.physics.location() + Displacement::EYE_HEIGHT;
        let faces = location.faces();
        let min_position = faces.iter().position_min_by_key(|loc| FloatOrd(loc.dist2(eye_loc))).unwrap();
        let look_location = faces[min_position];
        let face = Face::from(min_position as u8);

        Self {
            ticks,
            location,
            face,
            first: true,
            look_location,
        }
    }
}

impl TaskTrait for MineTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        if self.first {
            self.first = false;
            out.mine(self.location, Mine::Start, self.face);
        }

        local.physics.look_at(self.look_location);

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

pub struct LazyTask {
    inner: Option<Box<Task>>,
    create_task: Option<Box<dyn FnOnce(&mut LocalState, &GlobalState) -> Task + 'static + Sync + Send>>,
}

impl LazyTask {
    pub fn new<T: FnOnce(&mut LocalState, &GlobalState) -> Task + 'static + Sync + Send>(block: T) -> LazyTask {
        Self {
            inner: None,
            create_task: Some(box block),
        }
    }

    fn get(&mut self, local: &mut LocalState, global: &GlobalState) -> &mut Task {
        if self.inner.is_none() {
            let f = self.create_task.take().unwrap();
            self.inner = Some(Box::new(f(local, global)));
        }

        self.inner.as_mut().unwrap()
    }
}

impl TaskTrait for LazyTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let task = self.get(local, global);
        task.tick(out, local, global)
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        let task = self.get(local, global);
        task.expensive(end_at, local, global);
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
            place_loc: BlockLocation::from(local.physics.location()).below(),
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

pub struct BridgeTask {
    count: u32,
    place_against: BlockLocation,
    direction: CardinalDirection,
}

impl BridgeTask {
    pub fn new(count: u32, direction: CardinalDirection, local: &LocalState) -> BridgeTask {
        let start = BlockLocation::from(local.physics.location()).below();
        Self {count, place_against: start, direction}
    }
}

impl TaskTrait for BridgeTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {

        let displacement = Displacement::from(self.direction.unit_change());

        let direction = Direction::from(displacement);

        local.physics.look(direction);
        local.physics.line(Line::Forward);
        local.physics.jump();
        local.physics.speed(Speed::SPRINT);

        let target_loc = self.place_against.true_center();
        let current_loc = local.physics.location();

        let place = match self.direction {
            CardinalDirection::North => {
                target_loc.x - current_loc.x < (0.4 - 0.2)
            }
            CardinalDirection::South => {
                target_loc.x - current_loc.x > (-0.4 + 0.2)
            }
            CardinalDirection::West => {
                target_loc.z - current_loc.z > (0.4 - 0.2)
            }
            CardinalDirection::East => {
                target_loc.z + current_loc.z > (0.4 + 0.2)
            }
        };

        if place {
            let face = Face::from(self.direction);
            local.physics.place_hand_face(self.place_against, face);
            let change = BlockLocation::from(self.direction.unit_change());
            self.place_against = self.place_against + change;
            self.count -= 1;
        }

        self.count == 0

    }
}
