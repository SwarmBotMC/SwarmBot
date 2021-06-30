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

const MIN_JUMP_DIST: f64 = 1.2;
const MIN_SPRINT_DIST: f64 = 3.2;
const MAX_JUMP_DIST: f64 = 3.95;

const MAX_TICKS: usize = 29;

#[derive(Eq, PartialEq, Debug)]
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

    pub fn merge(&mut self, result: PathResult<MoveRecord>) {

        let other = match Follower::new(result) {
            Some(res) => res,
            None => return
        };

        let on = self.xs.front();

        let location_on = match on {
            None => {
                *self = other;
                return;
            },
            Some(val) => *val
        };


        let mut temp_xs = other.xs.clone();

        let mut idx = 0;

        const ITERS_UNTIL_FAIL: usize = 100;

        while let Some(&loc) = temp_xs.front() {
            idx += 1;
            if loc == location_on {
                *self = other;
                self.xs = temp_xs;
                return;
            }
            temp_xs.pop_front();

            if idx > ITERS_UNTIL_FAIL {
                break;
            }
        }

        *self = other;
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
        self.should_recalc
    }

    pub fn follow(&mut self, local: &mut LocalState, _global: &mut GlobalState) -> FollowResult {

        if !self.complete && !self.should_recalc && local.physics.on_ground() {
            let recalc = self.xs.len() * 2 < self.initial;
            if recalc {
                println!("recalc");
                self.should_recalc = true;
            }
        }

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


#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::time::{Duration, Instant};

    use crate::client::follow::{Follower, FollowResult};
    use crate::client::pathfind::implementations::novehicle::TravelProblem;
    use crate::client::pathfind::implementations::Problem;
    use crate::client::state::global::GlobalState;
    use crate::client::state::local::LocalState;
    use crate::client::timing::Increment;
    use crate::schematic::Schematic;
    use crate::storage::block::BlockLocation;
    use more_asserts::*;

    #[test]
    fn test_parkour_course() {
        let mut reader = OpenOptions::new()
            .read(true)
            .open("test-data/parkour.schematic")
            .unwrap();

        let course = Schematic::load(&mut reader);


        let mut local_state = LocalState::mock();
        let mut global_state = GlobalState::init();

        global_state.world_blocks.load(&course);

        let start = BlockLocation::new(-162, 82, -357);
        let end = BlockLocation::new(-152, 80, -338);

        let world = &global_state.world_blocks;
        let start_below = world.get_block(start.below()).unwrap().as_real().id();
        let end_below = world.get_block(end.below()).unwrap().as_real().id();

        // the ids of stained glass
        assert_eq!(95, start_below);
        assert_eq!(95, end_below);

        let mut problem = TravelProblem::new(start, end);

        let increment = problem.iterate_until(Instant::now() + Duration::from_secs(10), &mut local_state, &global_state);

        let result = match increment {
            Increment::InProgress => panic!("not finished"),
            Increment::Finished(res) => res
        };


        assert!(result.complete);

        let mut follower = Follower::new(result).unwrap();

        local_state.physics.teleport(start.center_bottom());

        while let FollowResult::InProgress = follower.follow(&mut local_state, &mut global_state) {
            local_state.physics.tick(&global_state.world_blocks);
            assert!(local_state.physics.location().y > 79.0, "the player fell... location was {}", local_state.physics.location());
        }

        assert_eq!(follower.follow(&mut local_state, &mut global_state), FollowResult::Finished);
        assert_lt!(local_state.physics.location().dist2(end.center_bottom()), 0.6 * 0.6);
    }


    #[test]
    fn test_bedrock() {

        let mut local_state = LocalState::mock();
        let mut global_state = GlobalState::init();

        let start = BlockLocation::new(0,1,0);
        let end = BlockLocation::new(950, 1, 950);

        let world = &mut global_state.world_blocks;
        world.set_random_floor();

        let mut problem = TravelProblem::new(start, end);

        let increment = problem.iterate_until(Instant::now() + Duration::from_secs(20), &mut local_state, &global_state);

        let result = match increment {
            Increment::InProgress => panic!("not finished"),
            Increment::Finished(res) => res
        };


        assert!(result.complete, "result is not complete. length was {}", result.value.len());

        let mut follower = Follower::new(result).unwrap();

        local_state.physics.teleport(start.center_bottom());

        while let FollowResult::InProgress = follower.follow(&mut local_state, &mut global_state) {
            local_state.physics.tick(&global_state.world_blocks);
            assert!(local_state.physics.location().y >= 0.0, "the player fell... location was {} front was {:?} left {}", local_state.physics.location(), follower.xs.front(), follower.xs.len());
        }

        assert_eq!(follower.follow(&mut local_state, &mut global_state), FollowResult::Finished);
        assert_lt!(local_state.physics.location().dist2(end.center_bottom()), 0.6 * 0.6);
    }
}
