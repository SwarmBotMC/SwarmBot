use std::collections::VecDeque;

use crate::client::pathfind::context::MoveRecord;
use crate::client::pathfind::incremental::PathResult;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::types::{Direction, Displacement, Location};

const PROGRESS_THRESHOLD: f64 = 0.15;
const PROGRESS_THRESHOLD_Y: f64 = 1.3;

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

    pub fn follow(&mut self, local: &mut LocalState, global: &mut GlobalState) -> FollowResult {
        if self.xs.is_empty() {
            return FollowResult::Finished;
        }

        self.ticks += 1;

        // more than 7 seconds on same block => failed
        if self.ticks >= 20 * 3 {
            println!("follower failed (time) for {} -> {}", local.physics.location(), self.xs.front().unwrap());
            return FollowResult::Failed;
        }

        let next = self.xs.front();

        let next = match next {
            None => return if self.complete { FollowResult::InProgress } else { FollowResult::Failed },
            Some(next) => *next
        };

        let current = local.physics.location();
        let displacement = next - current;

        let mag2_horizontal = Displacement::new(displacement.dx, 0.0, displacement.dz).mag2();

        // sqrt(2) is 1.41 which is the distance from the center of a block to the next
        if mag2_horizontal > 1.6 * 1.6 {
            println!("follower failed ... dist was {}", mag2_horizontal.sqrt());
            return FollowResult::Failed;
        }

        if mag2_horizontal < PROGRESS_THRESHOLD * PROGRESS_THRESHOLD && displacement.dy.abs() < PROGRESS_THRESHOLD_Y {
            self.next();
        }

        let mag2 = Displacement::new(displacement.dx, 0.0, displacement.dz).mag2();

        if mag2 < 0.01 * 0.01 {
            // want to avoid divide by 0 for direction
            return FollowResult::InProgress;
        }

        let dir = Direction::from(displacement);
        local.physics.look(dir);

        if displacement.dy > 0.0 {
            // we want to move vertically first (jump)
            local.physics.jump();
        } else if displacement.dy < 0.0 {
            // only will do anything if we are in water
            // local.physics.descend();
        }

        local.physics.line(Line::Forward);
        local.physics.speed(Speed::WALK);

        FollowResult::InProgress
    }
}
