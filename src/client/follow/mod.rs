use std::collections::VecDeque;

use crate::client::pathfind::context::MoveContext;
use crate::types::{Location, Displacement, Direction};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::physics::Walk;

#[derive(Eq, PartialEq)]
pub enum FollowResult {
    Failed,
    InProgress,
    Success
}

#[derive(Debug)]
pub struct Follower {
    xs: VecDeque<Location>,
}

impl Follower {
    pub fn new(path: Vec<MoveContext>) -> Follower {

        let xs = path.into_iter().map(|ctx| {
            let loc = ctx.location;
            Location::new(loc.0 as f64 + 0.5, loc.1 as f64, loc.2 as f64 + 0.5)
        }).collect();

        Follower {
            xs,
        }
    }
    pub fn follow(&mut self, local: &mut LocalState, global: &mut GlobalState) -> FollowResult {
        let next = self.xs.front();

        let next = match next {
            None => return FollowResult::Success,
            Some(next) => *next
        };

        let current = local.physics.location();
        let displacement = next - current;

        let mag2 = displacement.mag2();
        let mag2_horizontal = Displacement::new(displacement.dx, 0.0, displacement.dz).mag2();

        // sqrt(2) is 1.41 which is the distance from the center of a block to the next
        if mag2_horizontal > 1.3*1.3 {
            return FollowResult::Failed;
        }

        let res = if mag2 < 0.2 * 0.2 {
            self.xs.pop_front();
            FollowResult::Success
        } else {
            FollowResult::InProgress
        };

        if mag2 < 0.01 * 0.01 {
            // want to avoid divide by 0
            return FollowResult::Success;
        }

        let dir = Direction::from(displacement);
        local.physics.look(dir);

        if displacement.dy > 0.0 {
            local.physics.jump();
            // we want to move vertically first (jump)
        }

        local.physics.walk(Walk::Forward);

        return res;
    }
}
