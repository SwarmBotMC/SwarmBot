/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use std::collections::{VecDeque, HashSet};
use std::time::Instant;

use float_ord::FloatOrd;
use itertools::Itertools;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::{PlayerProblem, Problem};
use crate::client::pathfind::implementations::novehicle::{BlockGoalCheck, BlockHeuristic, CenterChunkGoalCheck, ChunkHeuristic, TravelChunkProblem, TravelProblem};
use crate::client::pathfind::moves::CardinalDirection;
use crate::client::pathfind::traits::{GoalCheck, Heuristic};
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::protocol::{Face, InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState, SimpleType};
use crate::storage::blocks::ChunkLocation;
use crate::types::{Direction, Displacement, Location};

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
    PillarAndMineTask,
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
    problem: Box<PlayerProblem<H, G>>,
    follower: Option<Follower>,
}


impl<H: Heuristic, G: GoalCheck> NavigateProblem<H, G> {
    fn raw(problem: PlayerProblem<H, G>) -> NavigateProblem<H, G> {
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

pub type ChunkTravelTask = NavigateProblem<ChunkHeuristic, CenterChunkGoalCheck>;
pub type BlockTravelTask = NavigateProblem<BlockHeuristic, BlockGoalCheck>;

impl ChunkTravelTask {
    pub fn new(goal: ChunkLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_center_chunk(start, goal);
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

    pub fn prepend(&mut self, task: impl Into<Task>) {
        self.tasks.push_front(task.into());
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

        true
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        match self.tasks.front_mut() {
            None => {}
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
        let kind = global.blocks.get_block_kind(location).unwrap();
        let tool = local.inventory.current_tool();

        let mut ticks = tool.wait_time(kind, false, true, &global.block_data);

        if ticks == 0 {
            ticks += 1;
        }

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
        local.physics.look_at(self.look_location);

        if self.first {
            out.swing_arm();
            self.first = false;
            out.mine(self.location, Mine::Start, self.face);
        }

        out.swing_arm();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, self.face);
            global.blocks.set_block(self.location, BlockState::AIR);
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}

pub trait TaskStream {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> Option<Task>;
}

pub struct LazyStream<T: TaskStream> {
    current: Option<Box<Task>>,
    create_task: T,
}

impl<T: TaskStream> LazyStream<T> {

    fn new(create_task: T) -> LazyStream<T> {
        Self {create_task, current: None}
    }

    fn get(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> Option<&mut Task> {
        if self.current.is_none() {
            let next = self.create_task.poll(out, local, global)?;
            self.current = Some(box next)
        }

        self.current.as_deref_mut()
    }
}

impl<T: TaskStream> TaskTrait for LazyStream<T> {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let current = match self.get(out, local, global) {
            None => return true,
            Some(inner) => inner
        };

        let finished_subtask = current.tick(out, local, global);
        if finished_subtask {
            self.current = None;
        }

        false
    }

    fn expensive(&mut self, end_by: Instant, local: &mut LocalState, global: &GlobalState) {
        let current = match self.current.as_mut() {
            None => return,
            Some(inner) => inner
        };
        current.expensive(end_by, local, global);
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


pub type PillarAndMineTask = LazyStream<PillarOrMine>;

impl PillarAndMineTask {
   pub fn pillar_and_mine(height: u32) -> Self {
       let state = PillarOrMine {height};
       Self::new(state)
   }
}

pub struct PillarOrMine {
    height: u32,
}

impl TaskStream for PillarOrMine {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> Option<Task> {

        if self.height == 0 {
            return None;
        }

        let above = local.physics.location() + Displacement::new(0., 3.5, 0.);
        let mut set = HashSet::new();
        local.physics.in_cross_section(above, &global.blocks, &mut set);
        if let Some(position) = set.into_iter().next() {
            local.inventory.switch_tool(out);
            Some(MineTask::new(position, local, global).into())
        } else {
            local.inventory.switch_block(out);
            self.height -= 1;
            Some(PillarTask::new(1, local).into())
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
            place_loc: BlockLocation::from(local.physics.location()).below(),
        }
    }
}

impl TaskTrait for PillarTask {
    fn tick(&mut self, _: &mut impl InterfaceOut, local: &mut LocalState, _: &mut GlobalState) -> bool {
        local.physics.jump();
        let down = Direction {
            yaw: 90.,
            pitch: 90.,
        };
        local.physics.look(down);

        if local.physics.location().y - self.place_loc.y as f64 > 2.18 {
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
                out.swing_arm();
                out.place_block(self.location.unwrap(), Face::PosY);
                return true;
            }
            return false;
        }

        let current_loc = local.physics.location();
        let below = global.blocks.first_below(current_loc.into());
        match below {
            None => {}
            Some((location, _)) => {
                local.physics.look_at(location.center_bottom());
                let dy = current_loc.y - (location.y as f64 + 1.0);
                if dy < 3.4 {
                    out.swing_arm();
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
        Self { count, place_against: start, direction }
    }
}

impl TaskTrait for BridgeTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let displacement = Displacement::from(self.direction.unit_change());

        let direction = Direction::from(-displacement);

        local.physics.look(direction);
        local.physics.line(Line::Backward);
        local.physics.speed(Speed::WALK);

        let target_loc = self.place_against.true_center();
        let current_loc = local.physics.location();

        let place = match self.direction {
            CardinalDirection::North => {
                target_loc.x - current_loc.x < (-0.6)
            }
            CardinalDirection::South => {
                target_loc.x - current_loc.x > (-0.4 + 0.5)
            }
            CardinalDirection::West => {
                target_loc.z - current_loc.z > (0.4 - 0.5)
            }
            CardinalDirection::East => {
                target_loc.z + current_loc.z > (0.4 + 0.5)
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
