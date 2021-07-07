/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

use float_ord::FloatOrd;
use itertools::Itertools;

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

pub type ChunkTravelTask = NavigateProblem<ChunkHeuristic, CenterChunkGoalCheck>;
pub type BlockTravelTask = NavigateProblem<BlockHeuristic, BlockGoalCheck>;
pub type BlockTravelNearTask = NavigateProblem<BlockHeuristic, BlockNearGoalCheck>;
pub type MineLayerTask = LazyStream<MineLayer>;
pub type MineColumnTask = LazyStream<MineColumn>;

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(TaskTrait)]
pub enum Task {
    EatTask,
    BridgeTask,
    MineColumnTask,
    MineTask,
    BlockTravelNearTask,
    PillarTask,
    DelayTask,
    PillarAndMineTask,
    BlockTravelTask,
    ChunkTravelTask,
    MineLayerTask,
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
    // pub fn mine_all<T: IntoIterator<Item=BlockLocation>>(blocks: T, local: &LocalState, global: &GlobalState) -> CompoundTask {
    //     let eye_location = local.physics.location() + Displacement::EYE_HEIGHT;
    //     let tasks = blocks
    //         .into_iter()
    //         .sorted_unstable_by_key(|&loc| FloatOrd(loc.true_center().dist2(eye_location)))
    //         .map(|loc| MineTask::new(loc, local, global).into())
    //         .collect();
    //
    //     Self {
    //         tasks
    //     }
    // }
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

impl CompoundTask {
    pub fn add<T: Into<Task>>(&mut self, task: T) -> &mut Self {
        self.tasks.push_back(task.into());
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
    pub fn new(location: BlockLocation, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> MineTask {
        let kind = global.blocks.get_block_kind(location).unwrap();
        let tool = local.inventory.switch_tool(out);

        let ticks = tool.wait_time(kind, false, true, &global.block_data) + 1;

        // if ticks > 1 {
        //     ticks -= 1;
        // }

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
            out.mine(self.location, Mine::Start, Face::PosY);
        }

        out.swing_arm();
        if self.ticks == 0 {
            out.mine(self.location, Mine::Finished, Face::PosY);
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

impl<T: TaskStream> From<T> for LazyStream<T> {
    fn from(create_task: T) -> Self {
        Self { create_task, current: None }
    }
}

impl<T: TaskStream> LazyStream<T> {
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
        while let Some(task) = self.get(out, local, global) {
            let finished_subtask: bool = task.tick(out, local, global);
            if finished_subtask {
                self.current = None;
            } else {
                return false;
            }
        }

        true
    }

    fn expensive(&mut self, end_by: Instant, local: &mut LocalState, global: &GlobalState) {
        let current = match self.current.as_mut() {
            None => return,
            Some(inner) => inner
        };
        current.expensive(end_by, local, global);
    }
}

pub struct LazyTask<T: Lazy> {
    inner: Option<Box<Task>>,
    create_task: Option<T>,
}

pub trait Lazy {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task;
}

impl<T: Lazy> LazyTask<T> {
    pub fn new(block: T) -> LazyTask<T> {
        Self {
            inner: None,
            create_task: Some(block),
        }
    }

    fn get(&mut self, local: &mut LocalState, global: &GlobalState) -> &mut Task {
        if self.inner.is_none() {
            let f = self.create_task.take().unwrap();
            self.inner = Some(Box::new(f.create(local, global)));
        }

        self.inner.as_mut().unwrap()
    }
}

impl<T: Lazy> TaskTrait for LazyTask<T> {
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
        let state = PillarOrMine { height };
        Self::from(state)
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
            Some(MineTask::new(position, out, local, global).into())
        } else {
            local.inventory.switch_block(out);
            self.height -= 1;
            Some(PillarTask::new(1).into())
        }
    }
}


pub struct PillarTask {
    dest_y: u32,
    jumped: bool,
}

impl PillarTask {
    pub fn new(dest_y: u32) -> PillarTask {
        Self {
            dest_y,
            jumped: false,
        }
    }
}

impl TaskTrait for PillarTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        local.inventory.switch_block(out);

        if local.physics.on_ground() && local.physics.location().y as u32 == self.dest_y {
            return true;
        }

        if local.physics.on_ground() {
            local.physics.jump();
            self.jumped = true;
        }

        let down = Direction {
            yaw: 90.,
            pitch: 90.,
        };

        local.physics.look(down);

        if local.physics.at_apex() && self.jumped {
            let below = BlockLocation::from(local.physics.location()).below();
            let against = below.below();
            if global.blocks.get_block_simple(against) == Some(SimpleType::Solid) {
                local.physics.place_hand(against);
            }
        }

        false
    }
}


#[derive(Default)]
pub struct FallBucketTask {
    placed: bool,
    ticks_since_place: u32,
    iter: bool,
    place_loc: Option<BlockLocation>,
}

impl TaskTrait for FallBucketTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        const BUCKET_LEAVE_TICKS: u32 = 5;

        if self.placed {
            self.ticks_since_place += 1;
            if self.ticks_since_place > BUCKET_LEAVE_TICKS {
                out.use_item();
                let place_loc = self.place_loc.unwrap();
                out.place_block(place_loc, Face::PosY);
                global.blocks.set_block(place_loc.above(), BlockState::AIR);
                return true;
            }
            return false;
        }

        let current_loc = local.physics.location();
        let below = global.blocks.first_below(BlockLocation::from(current_loc));
        match below {
            None => {}
            Some((location, _)) => {
                if !self.iter {
                    let height = local.physics.location().y;
                    if height - (location.y as f64 + 1.0) < 3.0 {
                        return true;
                    }
                } else {
                    local.inventory.switch_bucket(out);
                }

                local.physics.look_at(location.center_bottom());
                let dy = current_loc.y - (location.y as f64 + 1.0);
                if dy < 3.4 {
                    out.use_item();
                    out.place_block(location, Face::PosY);
                    global.blocks.set_block(location.above(), BlockState::WATER);
                    self.place_loc = Some(location);
                    self.placed = true;
                    self.ticks_since_place = 0;
                }
            }
        }

        self.iter = true;

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


pub struct MineColumn {
    on_loc: BlockLocation,
}

impl MineColumn {
    pub fn new(local: &LocalState) -> MineColumn {
        let on_loc = BlockLocation::from(local.physics.location()).below();
        Self { on_loc }
    }
}

impl TaskStream for MineColumn {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> Option<Task> {
        if self.on_loc.y > 5 {
            let mine_task = MineLayer::new(self.on_loc, global)?;
            let task = LazyStream::from(mine_task);
            let mut compound = CompoundTask::default();
            compound
                .add(task)
                .add(DelayTask(5))
                .add(FallBucketTask::default())
                .add(PillarTask::new(self.on_loc.y as u32));

            self.on_loc.y -= 1;

            Some(compound.into())
        } else {
            None
        }
    }
}

pub struct MineLayer {
    blocks_to_mine: VecDeque<BlockLocation>,
    start_loc: BlockLocation,
}

impl MineLayer {
    pub fn new(origin_loc: BlockLocation, global: &GlobalState) -> Option<MineLayer> {
        const RADIUS: u8 = 3;

        let blocks_to_mine = global.blocks
            .y_slice(origin_loc, RADIUS, |state| state.kind().mineable(&global.block_data))?
            .into_iter()
            .sorted_unstable_by_key(|&loc| {
                let priority = if loc == origin_loc {
                    // we always want to do our current loc last so we don't fall when mining other blocks
                    f64::INFINITY
                } else {
                    // else sort closest to furthes away
                    loc.dist2(origin_loc)
                };
                FloatOrd(priority)
            })
            .collect();


        Some(Self {
            blocks_to_mine,
            start_loc: origin_loc,
        })
    }
}

impl TaskStream for MineLayer {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &GlobalState) -> Option<Task> {
        self.blocks_to_mine
            .pop_front()
            .map(|loc| MineTask::new(loc, out, local, global).into())
    }
}
