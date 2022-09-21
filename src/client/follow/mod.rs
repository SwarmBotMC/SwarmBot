use std::collections::VecDeque;

use crate::{
    client::{
        pathfind::{context::MoveRecord, incremental::PathResult},
        physics::{speed::Speed, Line},
        state::{global::GlobalState, local::LocalState},
    },
    types::{Direction, Location},
};

const PROGRESS_THRESHOLD: f64 = 0.3;
const PROGRESS_THRESHOLD_Y: f64 = 0.48;

const EPSILON: f64 = 0.001;

const MIN_JUMP_DIST: f64 = 1.2;
const MIN_SPRINT_DIST: f64 = 3.0;
const MAX_JUMP_DIST: f64 = 4.0;

const MAX_TICKS: usize = 20 * 10;

#[derive(Eq, PartialEq, Debug)]
pub enum FollowResult {
    Failed,
    InProgress,
    Finished,
}

/// Given a path the follower decides which moves (analagous to keys a real
/// player would press) the bot should take. Currently the follower is totally
/// legit---it interfaces with the [Physics] struct which only allows for moves
/// a real player could do* * = probably not as precisely.
///
/// # Jumping
/// Jumping considers the velocity as well as the current displacement
/// which fixes many edge cases. Before the bot would likely fail if jumping
/// in a three block pattern where the path between them has a 90 degree edge.
/// This is because the bot's heading only focused on the next block---it
/// did not have a factor to counter the velocity going _away_ from the
/// current target.
#[derive(Debug)]
pub struct Follower {
    xs: VecDeque<Location>,
    initial: usize,
    ticks: usize,
    complete: bool,
    should_recalc: bool,
}

impl Follower {
    pub fn points(&self) -> &VecDeque<Location> {
        &self.xs
    }
    pub fn new(path_result: PathResult<MoveRecord>) -> Option<Follower> {
        let path = path_result.value;
        if path.is_empty() {
            return None;
        }

        let initial = path.len();
        let xs = path
            .into_iter()
            .map(|ctx| {
                let loc = ctx.state.location;
                loc.center_bottom()
            })
            .collect();

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
            None => return,
        };

        let on = self.xs.front();

        let location_on = match on {
            None => {
                *self = other;
                return;
            }
            Some(val) => *val,
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
        // We only want to recalc if we are on the ground to prevent issues with the
        // pathfinder thinking we are one block higher. We do this if the path
        // is incomplete and we have gone through at least half of the nodes
        if !self.complete && !self.should_recalc && local.physics.on_ground() {
            let recalc = self.xs.len() * 2 < self.initial;
            if recalc {
                self.should_recalc = true;
            }
        }

        let mut mag2_horizontal;
        let mut displacement;

        let current = local.physics.location();
        loop {
            let on = match self.xs.front() {
                None => {
                    return if self.complete {
                        FollowResult::Finished
                    } else {
                        FollowResult::Failed
                    }
                }
                Some(on) => *on,
            };

            displacement = on - current;
            mag2_horizontal = displacement.make_dy(0.).mag2();

            if mag2_horizontal < PROGRESS_THRESHOLD * PROGRESS_THRESHOLD
                && -EPSILON <= displacement.dy
                && displacement.dy <= PROGRESS_THRESHOLD_Y
            {
                self.next();
            } else {
                break;
            }
        }

        // by default move forward and sprint. Strafing is not needed; we can just
        // change the direction we look
        local.physics.line(Line::Forward);
        local.physics.speed(Speed::SPRINT);

        // We include a tick counter so we can determine if we have been stuck on a
        // movement for too long
        self.ticks += 1;

        // more than 1.5 seconds on same block => failed
        if self.ticks >= MAX_TICKS {
            println!(
                "follower failed (time) for {} -> {}",
                local.physics.location(),
                self.xs.front().unwrap()
            );
            return FollowResult::Failed;
        }

        let disp_horizontal = displacement.make_dy(0.);
        let velocity = local.physics.velocity().make_dy(0.);

        const VELOCITY_IMPORTANCE: f64 = 1.5;
        const DISPLACEMENT_CONSIDER_THRESH: f64 = 0.05;

        let look_displacement = disp_horizontal - velocity * VELOCITY_IMPORTANCE;
        let corr = velocity.normalize().dot(disp_horizontal.normalize());

        // let vel_percent2 = (velocity.mag2() / (SPRINT_BLOCKS_PER_TICK *
        // SPRINT_BLOCKS_PER_TICK)).min(1.0);

        // println!("vel percent {}", vel_percent2);

        // if displacement.mag2() < DISPLACEMENT_CONSIDER_THRESH *
        // DISPLACEMENT_CONSIDER_THRESH {     look_displacement =
        // disp_horizontal;     corr = 1.0;
        // }

        const THRESH_VEL: f64 = 3.0 / 20.;
        // const THRESH_VEL: f64 = 0.0;

        // sqrt(2) is 1.41 which is the distance from the center of a block to the next
        if local.physics.on_ground() && mag2_horizontal > MIN_JUMP_DIST * MIN_JUMP_DIST {
            // it is far away... we probably have to jump to it

            // min distance we can jump at
            if mag2_horizontal < MAX_JUMP_DIST * MAX_JUMP_DIST
                && corr > 0.95
                && velocity.mag2() > THRESH_VEL * THRESH_VEL
            {
                local.physics.jump();
            }

            // walk if close
            if mag2_horizontal < MIN_SPRINT_DIST * MIN_SPRINT_DIST
                && velocity.mag2() > THRESH_VEL * THRESH_VEL
            {
                local.physics.speed(Speed::WALK);
            }
        }

        let mut dir = Direction::from(look_displacement);
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
    use std::{
        fs::OpenOptions,
        time::{Duration, Instant},
    };

    use interfaces::types::BlockLocation;
    use more_asserts::*;

    use crate::{
        client::{
            follow::{FollowResult, Follower},
            pathfind::implementations::{novehicle::TravelProblem, Problem},
            state::{
                global::GlobalState,
                local::{inventory::PlayerInventory, LocalState},
            },
            timing::Increment,
        },
        schematic::Schematic,
    };

    #[test]
    fn test_parkour_course() {
        let mut reader = OpenOptions::new()
            .read(true)
            .open("test-data/parkour.schematic")
            .unwrap();

        let course = Schematic::load(&mut reader);

        let mut local_state = LocalState::mock();
        let mut global_state = GlobalState::init();

        global_state.blocks.paste(&course);

        let start = BlockLocation::new(-162, 82, -357);
        let end = BlockLocation::new(-152, 80, -338);

        let world = &global_state.blocks;
        let start_below = world.get_block(start.below()).unwrap().as_real().id();
        let end_below = world.get_block(end.below()).unwrap().as_real().id();

        // the ids of stained glass
        assert_eq!(95, start_below);
        assert_eq!(95, end_below);

        let mut problem = TravelProblem::navigate_block(start, end);

        let increment = problem.iterate_until(
            Instant::now() + Duration::from_secs(10),
            &mut local_state,
            &global_state,
        );

        let result = match increment {
            Increment::InProgress => panic!("not finished"),
            Increment::Finished(res) => res,
        };

        assert!(result.complete);

        let mut follower = Follower::new(result).unwrap();

        local_state.physics.teleport(start.center_bottom());

        while let FollowResult::InProgress = follower.follow(&mut local_state, &mut global_state) {
            local_state
                .physics
                .tick(&mut global_state.blocks, &PlayerInventory::default());
            assert!(
                local_state.physics.location().y > 79.0,
                "the player fell... location was {}",
                local_state.physics.location()
            );
        }

        assert_eq!(
            follower.follow(&mut local_state, &mut global_state),
            FollowResult::Finished
        );
        assert_lt!(
            local_state.physics.location().dist2(end.center_bottom()),
            0.6 * 0.6
        );
    }

    #[test]
    fn test_bedrock() {
        let mut local_state = LocalState::mock();
        let mut global_state = GlobalState::init();

        let start = BlockLocation::new(0, 1, 0);
        let end = BlockLocation::new(950, 1, 950);

        let world = &mut global_state.blocks;
        world.set_random_floor();

        let mut problem = TravelProblem::navigate_block(start, end);

        // so we can get determininistic tests
        problem.set_max_millis(u128::MAX);

        let increment = problem.iterate_until(
            Instant::now() + Duration::from_secs(20),
            &mut local_state,
            &global_state,
        );

        let result = match increment {
            Increment::InProgress => panic!("not finished"),
            Increment::Finished(res) => res,
        };

        assert!(
            result.complete,
            "result is not complete (took too much time?). length was {}. player loc was {}",
            result.value.len(),
            local_state.physics.location()
        );

        let mut follower = Follower::new(result).unwrap();

        local_state.physics.teleport(start.center_bottom());

        while let FollowResult::InProgress = follower.follow(&mut local_state, &mut global_state) {
            local_state
                .physics
                .tick(&mut global_state.blocks, &local_state.inventory);
            assert!(
                local_state.physics.location().y >= 0.0,
                "the player fell... location was {} front was {:?} left {}",
                local_state.physics.location(),
                follower.xs.front(),
                follower.xs.len()
            );
        }

        assert_eq!(
            follower.follow(&mut local_state, &mut global_state),
            FollowResult::Finished
        );
        assert_lt!(
            local_state.physics.location().dist2(end.center_bottom()),
            0.6 * 0.6
        );
    }
}
