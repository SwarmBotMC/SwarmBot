/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::cmp::max;
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::string::ParseError;
use std::time::Instant;

use float_ord::FloatOrd;
use itertools::Itertools;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::novehicle::{TravelChunkProblem, TravelProblem};
use crate::client::pathfind::implementations::Problem;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::physics::tools::{Material, Tool};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::{BlockTravelTask, ChunkTravelTask, CompoundTask, EatTask, FallBucketTask, MineTask, PillarTask, Task, TaskTrait, DelayTask, BridgeTask};
use crate::client::timing::Increment;
use crate::error::Res;
use crate::protocol::{EventQueue, Face, InterfaceOut, Mine};
use std::error::Error;
use crate::storage::block::{BlockKind, BlockLocation};
use crate::storage::blocks::ChunkLocation;
use crate::types::{Direction, Displacement};
use std::fmt::{Display, Formatter};
use crate::client::pathfind::moves::CardinalDirection;

#[derive(Default)]
pub struct ActionState {
    task: Option<Task>,
}

impl ActionState {
    pub fn schedule<T: Into<Task>>(&mut self, task: T) {
        self.task = Some(task.into());
    }
    pub fn clear(&mut self) {
        self.task = None;
    }
}

pub struct Bot<Queue: EventQueue, Out: InterfaceOut> {
    pub state: LocalState,
    pub actions: ActionState,
    pub queue: Queue,
    pub out: Out,
}

impl<Queue: EventQueue, Out: InterfaceOut> Bot<Queue, Out> {
    pub fn run_sync(&mut self, global: &mut GlobalState) {
        match self.actions.task.as_mut() {
            None => {}
            Some(task) => {
                if task.tick(&mut self.out, &mut self.state, global) {
                    self.actions.task = None;
                }
            }
        }

        if self.actions.task.is_none() {
            let mut vel = self.state.physics.velocity();
            vel.dy = 0.;
            if vel.mag2() > 0.01 {
                vel *= -1.;
                self.state.physics.look(Direction::from(vel));
                self.state.physics.speed(Speed::SPRINT);
                self.state.physics.line(Line::Forward);
            }
        }

        let actions = self.state.physics.tick(&mut global.world_blocks);

        let physics = &self.state.physics;
        self.out.teleport_and_look(physics.location(), physics.direction(), physics.on_ground());

        if let Some(place) = actions.block_placed.as_ref() {
            self.out.right_click();
            self.out.place_block(place.location, place.face);
        }

        self.state.ticks += 1;
    }
}


#[derive(Error, Debug)]
pub enum ProcessError {

    #[error(transparent)]
    Parse(#[from] ParseIntError),

    #[error(transparent)]
    Count(#[from] WrongArgCount),
}

#[derive(Debug)]
pub struct WrongArgCount {
    required: u32
}

impl std::error::Error for WrongArgCount{}

impl WrongArgCount {
    pub fn new(required: u32) -> Self {
        Self {
            required
        }
    }
}

impl Display for WrongArgCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("wrong arg count")
    }
}

/// Always returns None.
pub fn process_command(name: &str, args: &[&str], local: &mut LocalState, global: &mut GlobalState, actions: &mut ActionState, out: &mut impl InterfaceOut) -> Result<(), ProcessError> {

    // println! but bold
    macro_rules! msg {
            () => {{
                println!();
            }};
            ($($msg: expr),*) => {{
                let to_print_raw = format!($($msg),*);
                let to_print = ansi_term::Color::Black.bold().paint(to_print_raw).to_string();
                println!("{}", to_print);
            }};
        }

    macro_rules! chat {
            ($($msg: expr),*) => {{
                let to_print_raw = format!($($msg),*);
                out.send_chat(&to_print_raw);
            }};
        }

    match name {
        "pillar" => {
            if let [a] = args {
                let amount = a.parse()?;
                actions.schedule(PillarTask::new(amount, local));
            }
        }
        "bridge" => {
            if let [a] = args {
                let amount = a.parse()?;
                actions.schedule(BridgeTask::new(amount, CardinalDirection::North, local));
            }
        }
        "pillarc" => {
            let goal = ChunkLocation::try_from(args)?;
            let mut task = CompoundTask::default();

            task
                .add(ChunkTravelTask::new(goal, local))
                .add(DelayTask::new(10))
                .add_lazy(move |local, global| {
                    let column = global.world_blocks.get_real_column(goal).unwrap();
                    let highest_block = column.select_down(|x| x.kind() != BlockKind(0)).next().unwrap();
                    let highest_block = column.block_location(goal, highest_block);

                    let current_loc = BlockLocation::from(local.physics.location()).below();
                    let diff_y = max(highest_block.y - current_loc.y, 0) as u32;
                    PillarTask::new(diff_y, local)
                });

            println!("scheduled");
            actions.schedule(task);
        }
        "minec" => {
            let goal = ChunkLocation::try_from(args)?;
            let eye_loc = local.physics.location() + Displacement::EYE_HEIGHT;
            let column = global.world_blocks.get_real_column(goal).unwrap();
            let blocks = column.select_locs(goal, |state| state.kind().mineable(&global.block_data))
                .filter(|loc| loc.true_center().dist2(eye_loc) < 3.5*3.5);

            let task = CompoundTask::mine_all(blocks, local, global);
            actions.schedule(task);
        }
        "gotoc" => { // goto chunk
            if let [a, b] = args {
                let x = a.parse()?;
                let z = b.parse()?;
                let goal = ChunkLocation(x, z);
                actions.schedule(ChunkTravelTask::new(goal, local));
            }
        }
        "jump" => {
            local.physics.jump();
        }
        "health" => {
            chat!("/msg RevolutionNow Health: {}, Food: {}", local.health, local.food);
        }
        "follow" => {
            local.follow_closest = true;
        }
        "kys" => {
            // TODO: try to kill themself by fall damage/lava/etc
        }
        "eat" => {
            out.right_click();

            // shouldn't need to be 40 (32... but because of lag I guess it sometimes does)
            let eat_task = EatTask { ticks: 40 };
            actions.task = Some(eat_task.into());
        }
        "slot" => {
            if let [number] = args {
                let number: u8 = number.parse().unwrap();
                out.change_slot(number);
            }
        }
        "fall" => {
            let below = BlockLocation::from(local.physics.location()).below();

            let mine = MineTask::new(below, local, global);
            let fall = FallBucketTask::default();
            let mut compound = CompoundTask::default();
            compound.add(mine).add(fall);
            actions.schedule(compound);
        }
        "tool" => {
            local.inventory.switch_tool(out);
        }
        "drop" => {
            local.inventory.drop_hotbar(out);
        }
        "goto" => {
            if let [id] = args {
                let id: u32 = id.parse().unwrap();
                let kind = BlockKind::from(id);

                let loc = BlockLocation::from(local.physics.location());

                let closest = global.world_blocks.closest(loc, usize::MAX, |state| state.kind() == kind);

                if let Some(closest) = closest {
                    actions.schedule(BlockTravelTask::new(closest, local));
                } else {
                    msg!("There is no block {} by me", id);
                }
            }

            if let [a, b, c] = args {
                let x = a.parse()?;
                let y = b.parse()?;
                let z = c.parse()?;
                let dest = BlockLocation::new(x, y, z);
                actions.schedule(BlockTravelTask::new(dest, local));
            }
        }
        "stop" => {
            actions.task = None;
        }
        "loc" => {
            msg!("My location is {} in {}", local.physics.location(), local.dimension);
        }
        "state" => {
            if let [name] = args {
                if name == &local.info.username {
                    msg!("location {}", local.physics.location());
                    msg!("on ground {}", local.physics.on_ground());
                    let below_loc = BlockLocation::from(local.physics.location() - Displacement::EPSILON_Y);
                    msg!("below kind {:?}", global.world_blocks.get_block_kind(below_loc));
                    msg!("inventory slots {:?}", local.inventory.hotbar());
                }
            }
        }
        "get" => {
            if let [a, b, c] = args {
                let x = a.parse()?;
                let y = b.parse()?;
                let z = c.parse()?;
                let location = BlockLocation::new(x, y, z);

                msg!("The block is {:?}", global.world_blocks.get_block(location));
            }
        }
        "place" => {
            if let [a, b, c] = args {
                let x = a.parse()?;
                let y = b.parse()?;
                let z = c.parse()?;

                let origin = local.physics.location() + Displacement::EYE_HEIGHT;

                let location = BlockLocation::new(x, y, z);
                let faces = location.faces();
                let best_loc_idx = IntoIterator::into_iter(faces).position_min_by_key(|loc| FloatOrd(loc.dist2(origin))).unwrap();

                local.physics.look_at(faces[best_loc_idx]);
                out.right_click();
                out.place_block(location, Face::from(best_loc_idx as u8));
            }
        }
        "mine" => {
            let origin = local.physics.location() + Displacement::EYE_HEIGHT;

            let closest = global.world_blocks.closest_in_chunk(origin.into(), |state| state.kind().mineable(&global.block_data));

            if let Some(closest) = closest {
                let mine_task = MineTask::new(closest, local, global);
                actions.schedule(mine_task);
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn run_threaded(_: &rayon::Scope, local: &mut LocalState, actions: &mut ActionState, global: &GlobalState, end_by: Instant) {
    if let Some(task) = actions.task.as_mut() {
        task.expensive(end_by, local, global);
    }
}
