/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::collections::VecDeque;

use crate::client::pathfind::context::MoveRecord;
use crate::client::pathfind::incremental::PathResult;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::types::{Direction, Location};

const PROGRESS_THRESHOLD: f64 = 0.6;
const PROGRESS_THRESHOLD_Y: f64 = 0.48;

const MIN_JUMP_DIST: f64 = 1.5;
const MIN_SPRINT_DIST: f64 = 3.0;
const MAX_JUMP_DIST: f64 = 4.0;

const MAX_TICKS: usize = 29;

#[derive(Eq, PartialEq)]
pub enum FollowResult {
    Failed,
    InProgress,
    Finished,
}

#[derive(Debug)]
pub struct Follower {
    xs: VecDeque<Location>,
    initial: usize,
    ticks: usize,
    complete: bool,
    should_recalc: bool,
}

impl Follower {
    pub fn new(path_result: PathResult<MoveRecord>) -> Option<Follower> {
        let path = path_result.value;
        if path.len() <= 1 { return None; }

        let initial = path.len();
        let xs = path.into_iter().map(|ctx| {
            let loc = ctx.state.location;
            loc.center_bottom()
        }).collect();

        Some(Follower {
            xs,
            initial,
            ticks: 0,
            complete: path_result.complete,
            should_recalc: false,
        })
    }

    fn next(&mut self) {
        self.xs.pop_front();
        self.ticks = 0;
    }

    pub fn should_recalc(&mut self) -> bool {

        // we should only recalc if this is not complete
        if self.complete {
            return false;
        }
        // we should only return once
        if self.should_recalc {
            return false;
        }
        let recalc = self.xs.len() * 2 < self.initial;
        self.should_recalc = recalc;
        recalc
    }

    pub fn follow(&mut self, local: &mut LocalState, _global: &mut GlobalState) -> FollowResult {
        if self.xs.is_empty() {
            return FollowResult::Finished;
        }

        local.physics.line(Line::Forward);
        local.physics.speed(Speed::SPRINT);

        self.ticks += 1;

        // more than 1.5 seconds on same block => failed
        if self.ticks >= MAX_TICKS {
            println!("follower failed (time) for {} -> {}", local.physics.location(), self.xs.front().unwrap());
            return FollowResult::Failed;
        }

        let mag2_horizontal;
        let displacement;

        let current = local.physics.location();
        loop {
            let on = match self.xs.front() {
                None => return if self.complete { FollowResult::Finished } else { FollowResult::Failed },
                Some(on) => *on
            };
            let disp = on - current;
            let mut displacement_horiz = disp;
            displacement_horiz.dy = 0.;
            let a = displacement_horiz.mag2();
            if a < PROGRESS_THRESHOLD * PROGRESS_THRESHOLD && 0.0 <= disp.dy && disp.dy <= PROGRESS_THRESHOLD_Y {
                self.next();
            } else {
                mag2_horizontal = a;
                displacement = disp;
                break;
            }
        }


        // sqrt(2) is 1.41 which is the distance from the center of a block to the next
        if local.physics.on_ground() && mag2_horizontal > MIN_JUMP_DIST * MIN_JUMP_DIST {
            // it is far away... we probably have to jump to it

            if mag2_horizontal < MAX_JUMP_DIST * MAX_JUMP_DIST {
                local.physics.jump();
            }

            if mag2_horizontal < MIN_SPRINT_DIST * MIN_SPRINT_DIST {
                local.physics.speed(Speed::WALK);
            }

            // so we can run before we jump

            // if mag2_horizontal > JUMP_EDGE_DIST * JUMP_EDGE_DIST {
            //     // if local.physics.on_edge() {
            //         local.physics.jump();
            //     // }
            // } else {
            //     local.physics.jump();
            // }
        }

        let mut dir = Direction::from(displacement);
        dir.pitch = 0.;
        local.physics.look(dir);

        if displacement.dy > 0.0 {
            // we want to move vertically first (jump)
            local.physics.jump();
        } else if displacement.dy < 0.0 {
            // only will do anything if we are in water
            // local.physics.descend();
        }

        FollowResult::InProgress
    }
}
