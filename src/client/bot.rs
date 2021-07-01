/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::time::Instant;

use float_ord::FloatOrd;
use itertools::Itertools;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::Problem;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::physics::tools::{Material, Tool};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::{EatTask, MineTask, Task, TaskTrait};
use crate::client::timing::Increment;
use crate::protocol::{EventQueue, Face, InterfaceOut, Mine};
use crate::storage::block::{BlockKind, BlockLocation};
use crate::types::{Direction, Displacement};
use crate::client::pathfind::implementations::novehicle::TravelProblem;

type Prob = Box<dyn Problem<Node=MoveNode>>;

#[derive(Default)]
pub struct ActionState {
    pub task: Option<Task>,
    pub follower: Option<Follower>,
    pub travel_problem: Option<Prob>,
    pub last_problem: Option<Prob>,
}

impl ActionState {
    pub fn travel_to_block(&mut self, goal: BlockLocation, local: &LocalState) {
        let from = local.physics.location().into();
        println!("starting nav");
        let problem = box TravelProblem::create(from, goal);

        self.travel_problem = Some(problem);
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

        self.move_around(global);

        if self.actions.follower.is_none() {
            let mut vel = self.state.physics.velocity();
            vel.dy = 0.;
            if vel.mag2() > 0.01 {
                vel *= -1.;
                self.state.physics.look(Direction::from(vel));
                self.state.physics.speed(Speed::SPRINT);
                self.state.physics.line(Line::Forward);
            }
        }

        self.state.physics.tick(&global.world_blocks);

        let physics = &self.state.physics;
        self.out.teleport_and_look(physics.location(), physics.direction(), physics.on_ground());

        self.state.ticks += 1;
    }

    fn move_around(&mut self, global: &mut GlobalState) {
        if let Some(follower) = self.actions.follower.as_mut() {
            let follow_result = follower.follow(&mut self.state, global);
            if follow_result == FollowResult::Failed || follower.should_recalc() {
                if let Some(mut problem) = self.actions.last_problem.take() {
                    let block_loc = self.state.physics.location().into();
                    problem.recalc(MoveNode::simple(block_loc));
                    self.actions.travel_problem = Some(problem);
                }

                if follow_result == FollowResult::Failed {
                    self.actions.follower = None;
                }
            } else if follow_result == FollowResult::Finished {
                self.actions.follower = None;
                self.actions.last_problem = None;
                self.actions.travel_problem = None;
            }
        } else if self.state.follow_closest {
            let current_loc = self.state.physics.location();
            let closest = global.world_entities.iter().min_by_key(|(_id, data)| {
                FloatOrd(data.location.dist2(current_loc))
            });

            if let Some((_id, data)) = closest {
                let displacement = data.location - current_loc;
                if displacement.has_length() {
                    let dir = Direction::from(displacement);
                    self.state.physics.look(dir);
                    self.state.physics.line(Line::Forward);
                    self.state.physics.speed(Speed::WALK);
                }
            }
        }
    }
}

pub fn process_command(name: &str, args: &[&str], local: &mut LocalState, global: &mut GlobalState, actions: &mut ActionState, out: &mut impl InterfaceOut) {

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
        "goto" => {
            if let [id] = args {
                let id: u32 = id.parse().unwrap();
                let kind = BlockKind::from(id);

                let loc = BlockLocation::from(local.physics.location());

                let closest = global.world_blocks.closest(loc, usize::MAX, |state| state.kind() == kind);

                if let Some(closest) = closest {
                    actions.travel_to_block(closest, local);
                } else {
                    msg!("There is no block {} by me", id);
                }
            }

            if let [a, b, c] = args {
                let x = a.parse().unwrap();
                let y = b.parse().unwrap();
                let z = c.parse().unwrap();
                let dest = BlockLocation::new(x, y, z);
                actions.travel_to_block(dest, local);
            }
        }
        "stop" => {
            actions.follower = None;
            actions.travel_problem = None;
            actions.last_problem = None;
        }
        "loc" => {
            msg!("My location is {} in {}", local.physics.location(), local.dimension);
        }
        "state" => {
            if let [name] = args {
                if name == &local.info.username {
                    msg!("location {}", local.physics.location());
                    msg!("on ground {}", local.physics.on_ground());
                    if let Some(follower) = actions.follower.as_ref() {
                        for point in follower.points() {
                            msg!("{}", point);
                        }
                    }
                }
            }
        }
        "get" => {
            if let [a, b, c] = args {
                let x = a.parse().unwrap();
                let y = b.parse().unwrap();
                let z = c.parse().unwrap();
                let location = BlockLocation::new(x, y, z);

                msg!("The block is {:?}", global.world_blocks.get_block(location));
            }
        }
        "place" => {
            if let [a, b, c] = args {
                let x = a.parse().unwrap();
                let y = b.parse().unwrap();
                let z = c.parse().unwrap();

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
                actions.task = Some(mine_task.into());
            }
        }
        _ => {
            // self.out.send_chat("invalid command");
        }
    }
}

pub fn run_threaded(_scope: &rayon::Scope, local: &mut LocalState, actions: &mut ActionState, global: &GlobalState, end_by: Instant) {

    // TODO: this is pretty jank
    if let Some(traverse) = actions.travel_problem.as_mut() {
        let res = traverse.iterate_until(end_by, local, global);

        if let Increment::Finished(res) = res {
            if !res.complete {
                println!("incomplete goal of size {}", res.value.len());
            }

            match actions.follower.as_mut() {
                None => actions.follower = {
                    println!("no merge");
                    Follower::new(res)
                },
                Some(follow) => {
                    println!("merging");
                    follow.merge(res)
                }
            }

            actions.last_problem = actions.travel_problem.take();
        }
    }
}
