use std::collections::VecDeque;
use crate::client::pathfind::context::MoveContext;
use crate::protocol::{InterfaceOut};
use crate::storage::block::BlockLocation;
use crate::types::Location;
use crate::client::state::local::LocalState;

#[derive(Debug)]
pub struct Follower {
    head: MoveContext,
    xs: VecDeque<MoveContext>
}
impl Follower {
    pub fn new(path: Vec<MoveContext>) -> Option<Follower> {
        let mut xs = VecDeque::from(path);
        xs.pop_front().map(|head| Follower {
                head,
                xs
            })
    }
    pub fn follow(&mut self, _state: &LocalState, out: &mut impl InterfaceOut){
        let next = self.xs.pop_front();
        let next = match next {
            None => return,
            Some(next) => next
        };

        let BlockLocation(x,y,z) = self.head.location;
        let BlockLocation(x_new,y_new,z_new) = next.location;

        let new_loc = Location {
            x: x_new as f64,
            y: y_new as f64,
            z: z_new as f64
        };

        println!("teleport {:?}", new_loc);
        out.teleport(new_loc)
    }
}
