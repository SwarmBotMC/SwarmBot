/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

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
use crate::client::tasks::{BlockTravelTask, ChunkTravelTask, CompoundTask, EatTask, FallBucketTask, MineTask, PillarTask, Task, TaskTrait};
use crate::client::timing::Increment;
use crate::error::Res;
use crate::protocol::{EventQueue, Face, InterfaceOut, Mine};
use crate::storage::block::{BlockKind, BlockLocation};
use crate::storage::blocks::ChunkLocation;
use crate::types::{Direction, Displacement};

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

        let actions = self.state.physics.tick(&global.world_blocks);

        let physics = &self.state.physics;
        self.out.teleport_and_look(physics.location(), physics.direction(), physics.on_ground());

        if let Some(place) = actions.block_placed.as_ref() {
            self.out.right_click();
            self.out.place_block(place.location, place.face);
        }

        self.state.ticks += 1;
    }
}


/// Always returns None.
pub fn process_command(name: &str, args: &[&str], local: &mut LocalState, global: &mut GlobalState, actions: &mut ActionState, out: &mut impl InterfaceOut) -> Result<(), ParseIntError> {

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
            actions.schedule(PillarTask::new(10, local));
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
            let kind = global.world_blocks.get_block_kind(below).unwrap();
            let tool = Tool::new(Material::HAND);
            let ticks = tool.wait_time(kind, false, true, &global.block_data) + 1;
            println!("mine ticks {}", ticks);

            out.mine(below, Mine::Start, Face::PosY);
            out.left_click();

            local.physics.look_at(below.center_bottom());
            let mine = MineTask { ticks, location: below, face: Face::PosY };
            let fall = FallBucketTask::default();
            let compound = CompoundTask::new([mine.into(), fall.into()]);
            actions.task = Some(compound.into())
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
                let kind = global.world_blocks.get_block_kind(closest).unwrap();
                let faces = closest.faces();

                println!("faces {:?}", faces);
                let best_loc_idx = IntoIterator::into_iter(faces).position_min_by_key(|loc| FloatOrd(loc.dist2(origin))).unwrap();

                let best_loc = faces[best_loc_idx];
                let face = Face::from(best_loc_idx as u8);

                let displacement = best_loc - origin;
                local.physics.look(displacement.into());

                let tool = Tool::new(Material::DIAMOND);
                let ticks = tool.wait_time(kind, false, true, &global.block_data) + 1;

                msg!("started mining at {} .. ticks {}.. face {:?}", closest, ticks, face);

                out.mine(closest, Mine::Start, face);
                out.left_click();

                let mine_task = MineTask { ticks, location: closest, face };
                actions.schedule(mine_task);
            }
        }
        _ => {}
    }

    return Ok(());
}

pub fn run_threaded(_: &rayon::Scope, local: &mut LocalState, actions: &mut ActionState, global: &GlobalState, end_by: Instant) {
    if let Some(task) = actions.task.as_mut() {
        task.expensive(end_by, local, global);
    }
}
