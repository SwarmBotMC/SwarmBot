use std::collections::VecDeque;

use crate::client::pathfind::context::MoveContext;
use crate::client::state::local::LocalState;
use crate::protocol::InterfaceOut;
use crate::types::{Displacement, Location};

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
    pub fn follow(&mut self, local: &mut LocalState, out: &mut impl InterfaceOut) {
        let next = self.xs.front();

        let next = match next {
            None => return,
            Some(next) => next
        };

        let current = local.location;

        let Displacement { dx, dy, dz } = *next - current;

        if dy > 0.0 {
            // we want to move vertically first (jump)
        } else if dy < 0.0 {
            // we want to move horizontally first
        } else {
            // no change in height
        }

        let to_loc = *next;

        out.teleport(to_loc);
        local.location = to_loc;
    }
}
