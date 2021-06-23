use std::collections::VecDeque;

use crate::client::pathfind::context::MoveContext;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::protocol::InterfaceOut;
use crate::types::{Direction, Location};
use crate::client::physics::Walk;

#[derive(Debug)]
pub struct Follower {
    head: Location,
    xs: VecDeque<Location>,
}

impl Follower {
    pub fn new(path: Vec<MoveContext>) -> Option<Follower> {
        let mut xs: VecDeque<_> = path.into_iter().map(|ctx| {
            let loc = ctx.location;
            Location::new(loc.0 as f64 + 0.5, loc.1 as f64, loc.2 as f64 + 0.5)
        }).collect();

        let head = xs.pop_front()?;
        Some(Follower {
            head,
            xs,
        })
    }
    pub fn follow(&mut self, local: &mut LocalState, global: &mut GlobalState, out: &mut impl InterfaceOut) {
        let next = self.xs.front();

        let next = match next {
            None => return,
            Some(next) => *next
        };

        let current = local.physics.location();
        let displacement = next - current;

        let mag2 = displacement.mag2();

        if mag2 < 0.2 * 0.2 {
            self.xs.pop_front();
        }

        if mag2 < 0.01 * 0.01 {
            // want to avoid divide by 0
            return;
        }

        let dir = Direction::from(displacement);
        local.physics.look(dir);

        if displacement.dy > 0.0 {
            local.physics.jump();
            // we want to move vertically first (jump)
        }

        local.physics.walk(Walk::Forward);
    }
}
