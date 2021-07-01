/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::time::Instant;

use float_ord::FloatOrd;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::MoveNode;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::physics::tools::{Material, Tool};
use crate::client::state::global::GlobalState;
use crate::client::state::local::{LocalState, Task, TaskKind};
use crate::client::timing::Increment;
use crate::protocol::{EventQueue, InterfaceOut, Mine};
use crate::storage::block::{BlockKind, BlockLocation};
use crate::types::Direction;

pub struct Bot<Queue: EventQueue, Out: InterfaceOut> {
    pub state: LocalState,
    pub queue: Queue,
    pub out: Out,
}

impl<Queue: EventQueue, Out: InterfaceOut> Bot<Queue, Out> {
    pub fn run_sync(&mut self, global: &mut GlobalState) {
        match self.state.task.as_mut() {
            None => {}
            Some(task) => {
                match task.kind {
                    TaskKind::Mine(loc) => {
                        self.out.left_click();
                        if task.ticks == 0 {
                            self.out.mine(loc, Mine::Finished);
                        }
                    }
                    TaskKind::Eat => {
                        // self.out.right_click();
                        if task.ticks == 0 {
                            println!("finish eating");
                            self.out.finish_eating();
                        }
                    }
                }


                if task.ticks == 0 {
                    self.state.task = None;
                } else {
                    task.ticks -= 1;
                }
            }
        }
        self.move_around(global);

        if self.state.follower.is_none() {
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
        if let Some(mut follower) = self.state.follower.take() {
            let follow_result = follower.follow(&mut self.state, global);
            if follow_result == FollowResult::Failed || follower.should_recalc() {
                if let Some(mut problem) = self.state.last_problem.take() {
                    let block_loc = self.state.physics.location().into();
                    problem.recalc(MoveNode::simple(block_loc));
                    self.state.travel_problem = Some(problem);
                }

                if follow_result == FollowResult::Failed {
                    self.state.follower = None;
                } else {
                    self.state.follower = Some(follower);
                }
            } else if follow_result == FollowResult::Finished {
                self.state.follower = None;
                self.state.last_problem = None;
                self.state.travel_problem = None;
            } else {
                self.state.follower = Some(follower);
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

pub fn process_command(name: &str, args: &[&str], local: &mut LocalState, global: &mut GlobalState, out: &mut impl InterfaceOut) {

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
            local.task = Some(Task {
                ticks: 40,
                kind: TaskKind::Eat,
            })
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

                let closest = global.world_blocks.closest(loc, |state| state.kind() == kind);

                if let Some(closest) = closest {
                    local.travel_to_block(closest);
                } else {
                    msg!("There is no block {} by me", id);
                }
            }

            if let [a, b, c] = args {
                let x = a.parse().unwrap();
                let y = b.parse().unwrap();
                let z = c.parse().unwrap();
                let dest = BlockLocation::new(x, y, z);
                local.travel_to_block(dest);
            }
        }
        "stop" => {
            local.travel_problem = None;
            local.last_problem = None;
        }
        "loc" => {
            msg!("My location is {} in {}", local.physics.location(), local.dimension);
        }
        "state" => {
            if let [name] = args {
                if name == &local.info.username {
                    msg!("location {}", local.physics.location());
                    if let Some(follower) = local.follower.as_ref() {
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
        "mine" => {
            if let [id] = args {
                let id: u32 = id.parse().unwrap();
                let kind = BlockKind::from(id);

                let origin = BlockLocation::from(local.physics.location());
                let closest = global.world_blocks.closest(origin, |state| state.kind() == kind);

                if let Some(closest) = closest {
                    let dir = closest.center_bottom() - origin.center_bottom();
                    local.physics.look(dir.into());

                    let tool = Tool::new(Material::DIAMOND);
                    let ticks = tool.wait_time(kind, false, true, &global.block_data);

                    msg!("started mining at {} .. ticks {}", closest, ticks);

                    let task = Task {
                        ticks,
                        kind: TaskKind::Mine(closest),
                    };

                    local.task = Some(task);
                    out.mine(closest, Mine::Start);
                }
            }
        }
        _ => {
            // self.out.send_chat("invalid command");
        }
    }
}

pub fn run_threaded(_scope: &rayon::Scope, local: &mut LocalState, global: &GlobalState, end_by: Instant) {

    // TODO: this is pretty jank
    if let Some(mut traverse) = local.travel_problem.take() {
        let res = traverse.iterate_until(end_by, local, global);

        if let Increment::Finished(res) = res {
            if !res.complete {
                println!("incomplete goal of size {}", res.value.len());
            }

            match local.follower.as_mut() {
                None => local.follower = {
                    println!("no merge");
                    Follower::new(res)
                },
                Some(follow) => {
                    println!("merging");
                    follow.merge(res)
                }
            }

            // we are done finding the path
            local.last_problem = Some(traverse);
        } else {
            local.travel_problem = Some(traverse)
        }
    }
}
