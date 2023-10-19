use std::collections::VecDeque;

use crate::{
    client::{
        pathfind::{context::MoveRecord, incremental::PathResult},
        physics::{speed::Speed, Line},
        state::{global::GlobalState, local::LocalState},
    },
    types::{Direction, Location},
};

/// the threshold we need to progress when in horizontal blocks from our target
///
/// Targets are centered horizontally at the top surface of a block
const HORIZONTAL_PROGRESS_THRESHOLD: f64 = 0.3;

/// the maximum we can be below a block to progress. This is useful when
/// climbing ladders and swimming
const PROGRESS_MAX_BELOW_BLOCK: f64 = 0.48;

/// the y level we must achieve to complete a block
/// for instance this makes us not finish a block if we are still jumping
///
/// This is important because we want to be able to touch down to correct our
/// velocity before we start running towards the next block
const PROGRESS_MAX_ABOVE_BLOCK: f64 = 0.001;

/// the minimum distance we can jump from
const MIN_JUMP_DIST: f64 = 1.2;

/// the minimum distance we must be from a block to sprint
const MIN_SPRINT_DIST: f64 = 3.0;

/// maximum distance we can jump in blocks
const MAX_JUMP_DIST: f64 = 4.0;

/// the maximum number of ticks we can try progressing to another block
///
/// `20*10` means 10 seconds to progress between one jump
const MAX_PROGRESS_TICKS: usize = 20 * 10;

/// The result of trying to follow a path
#[derive(Eq, PartialEq, Debug)]
pub enum Result {
    /// we failed. we didn't get to the destination block we wanted to.
    /// maybe we fell or died or got stuck.
    Failed,

    /// we are in progress of moving following a path
    InProgress,

    /// we have finished following a path
    Finished,
}

/// Given a path the follower decides which moves (analogous to keys a real
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
    /// the locations we still need to reach
    xs: VecDeque<Location>,

    /// the initial path length
    initial: usize,

    /// the number of ticks since we gone to the next [`BlockLocation`] in the
    /// [`PathResult`]
    ticks: usize,

    /// if the path we were given was complete or not. If it is not, we will
    /// need to recalculate a path before we finish
    complete: bool,

    /// if we specifically think we should recalculate
    should_recalculate: bool,
}

impl Follower {
    /// Create a new [`Follower`]. If the `path_result` is empty, we will return
    /// [`None`]
    pub fn new(path_result: PathResult<MoveRecord>) -> Option<Self> {
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

        Some(Self {
            xs,
            initial,
            ticks: 0,
            complete: path_result.complete,
            should_recalculate: false,
        })
    }

    /// attempt to merge the current path with another path result.
    ///
    /// This can be done when we are using an incremental on-the-fly version of
    /// A* and we want to merge two path results.
    ///
    /// The merging is done by finding a path where two blocks touch each other.
    /// If there are no blocks that are shared between the current path and
    /// the calculate path the paths are not merged
    pub fn merge(&mut self, result: PathResult<MoveRecord>) {
        /// the number of iterators until we fail the merge task. Generally, the
        /// re-calculate path should not take a long time to compute, so
        /// this number can be fairly small.
        ///
        /// Generally paths will converge within maximally 3 or 4 blocks
        const ITERS_UNTIL_FAIL: usize = 100;

        let Some(other) = Self::new(result) else {
            return;
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

    /// go to the next point on the path
    fn next(&mut self) {
        self.xs.pop_front();
        self.ticks = 0;
    }

    /// if we should recalcualte the path
    pub fn should_recalc(&mut self) -> bool {
        // we should only recalc if this is not complete
        if self.complete {
            return false;
        }
        // we should only return once
        self.should_recalculate
    }

    /// an iteration where we attempt to stay on the given path
    pub fn follow_iteration(
        &mut self,
        local: &mut LocalState,
        _global: &mut GlobalState,
    ) -> Result {
        // We only want to recalculate if we are on the ground to prevent issues with
        // the pathfinder thinking we are one block higher. We do this if the
        // path is incomplete and we have gone through at least half of the
        // nodes
        if !self.complete && !self.should_recalculate && local.physics.on_ground() {
            let recalc = self.xs.len() * 2 < self.initial;
            if recalc {
                self.should_recalculate = true;
            }
        }

        let mut mag2_horizontal;
        let mut displacement;

        let current = local.physics.location();
        loop {
            let on = match self.xs.front() {
                None => {
                    return if self.complete {
                        Result::Finished
                    } else {
                        Result::Failed
                    }
                }
                Some(on) => *on,
            };

            displacement = on - current;
            mag2_horizontal = displacement.make_dy(0.).mag2();

            if mag2_horizontal < HORIZONTAL_PROGRESS_THRESHOLD * HORIZONTAL_PROGRESS_THRESHOLD
                && -PROGRESS_MAX_ABOVE_BLOCK <= displacement.dy
                && displacement.dy <= PROGRESS_MAX_BELOW_BLOCK
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
        if self.ticks >= MAX_PROGRESS_TICKS {
            println!(
                "follower failed (time) for {} -> {:?}",
                local.physics.location(),
                self.xs.front()
            );
            return Result::Failed;
        }

        let displacement_horizontal = displacement.make_dy(0.);
        let velocity = local.physics.velocity().make_dy(0.);

        /// how much we factor in velocity into how much we need to counter our
        /// look direction
        ///
        /// Suppose this player is you trying to get to the block `[]` and the
        /// current velocity is in the direction of `/`
        /// ```
        ///          /
        ///         o
        ///        \|/    ------->   []
        ///        /\
        /// ```
        ///
        /// If [`VELOCITY_IMPORTANCE`] is 0, the player will look directly at
        /// the block
        ///
        /// ```
        /// 
        ///         o ----
        ///        \|/    ------->   []
        ///        /\
        /// ```
        ///
        /// In the case where it is tuned correctly, the direction the player is
        /// looking should counter act the velocity so during a job it
        /// will actually hit the block. For instance, with `1.5` the
        /// player might reach
        ///
        /// ```
        /// 
        ///          o
        ///        \|/ \   ------->   []
        ///        /\   \
        /// ```
        const VELOCITY_IMPORTANCE: f64 = 1.5;

        let look_displacement = displacement_horizontal - velocity * VELOCITY_IMPORTANCE;

        // the correlation between the horizontal displacement and velocity
        let correlation = velocity
            .normalize()
            .dot(displacement_horizontal.normalize());

        /// the threshold velocity (in blocks per tick) that we need to achieve
        /// to perform a jump
        const JUMP_THRESHOLD_VEL: f64 = 3.0 / 20.;

        // sqrt(2) is 1.41 which is the distance from the center of a block to the next
        if local.physics.on_ground() && mag2_horizontal > MIN_JUMP_DIST * MIN_JUMP_DIST {
            // it is far away... we probably have to jump to it

            // if the horizontal distance is jumpable and our velocity is correct and we
            // have enough velocity to jump, jump
            if mag2_horizontal < MAX_JUMP_DIST * MAX_JUMP_DIST
                && correlation > 0.95
                && velocity.mag2() > JUMP_THRESHOLD_VEL * JUMP_THRESHOLD_VEL
            {
                local.physics.jump();
            }

            // walk if close (so we do not overshoot)
            if mag2_horizontal < MIN_SPRINT_DIST * MIN_SPRINT_DIST
                && velocity.mag2() > JUMP_THRESHOLD_VEL * JUMP_THRESHOLD_VEL
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

        Result::InProgress
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::OpenOptions,
        time::{Duration, Instant},
    };

    use anyhow::Context;
    use interfaces::types::BlockLocation;
    use more_asserts::assert_lt;

    use crate::{
        client::{
            follow::{Follower, Result},
            pathfind::implementations::{no_vehicle::TravelProblem, Problem},
            state::{
                global::GlobalState,
                local::{inventory::PlayerInventory, LocalState},
            },
            timing::Increment,
        },
        schematic::Schematic,
    };

    #[test]
    fn test_parkour_course() -> anyhow::Result<()> {
        let mut reader = OpenOptions::new()
            .read(true)
            .open("test-data/parkour.schematic")
            .context("could not open the parkour schematic")?;

        let course = Schematic::load(&mut reader).unwrap();

        let mut local_state = LocalState::mock();
        let mut global_state = GlobalState::init();

        global_state.blocks.paste(&course);

        let start = BlockLocation::new(-162, 82, -357);
        let end = BlockLocation::new(-152, 80, -338);

        let world = &global_state.blocks;
        let start_below = world
            .get_block(start.below())
            .context("could not get below block")?
            .as_real()
            .id();
        let end_below = world
            .get_block(end.below())
            .context("could not get end below block")?
            .as_real()
            .id();

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
            Increment::InProgress => anyhow::bail!("not finished"),
            Increment::Finished(res) => res,
        };

        assert!(result.complete);

        let mut follower = Follower::new(result).context("could not create a follower")?;

        local_state.physics.teleport(start.center_bottom());

        while let Result::InProgress =
            follower.follow_iteration(&mut local_state, &mut global_state)
        {
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
            follower.follow_iteration(&mut local_state, &mut global_state),
            Result::Finished
        );
        assert_lt!(
            local_state.physics.location().dist2(end.center_bottom()),
            0.6 * 0.6
        );

        Ok(())
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

        // so we can get deterministic tests
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

        while let Result::InProgress =
            follower.follow_iteration(&mut local_state, &mut global_state)
        {
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
            follower.follow_iteration(&mut local_state, &mut global_state),
            Result::Finished
        );
        assert_lt!(
            local_state.physics.location().dist2(end.center_bottom()),
            0.6 * 0.6
        );
    }
}
