use std::collections::VecDeque;
use crate::client::pathfind::context::MoveContext;
use crate::protocol::McProtocol;
use crate::storage::block::BlockLocation;
use crate::types::Location;
use crate::client::state::local::State;

pub struct Follower {
    head: MoveContext,
    xs: VecDeque<MoveContext>
}
impl Follower {
    pub fn new(path: Vec<MoveContext>) -> Option<Follower> {
        let mut xs = VecDeque::from(path);
        match xs.pop_front() {
            None => None,
            Some(head) => Some(Follower {
                head,
                xs
            })
        }
    }
    pub fn follow(&mut self, state: &State, protocol: &mut impl McProtocol){
        let next = self.xs.pop_front();
        let next = match next {
            None => return,
            Some(next) => next
        };

        let BlockLocation(x,y,z) = self.head.location;
        let BlockLocation(x_new,y_new,z_new) = next.location;

        let (dx, dy, dz) = (x_new - x, y_new - y, z_new -z);
        let new_loc = Location {
            x: x_new as f64,
            y: y_new as f64,
            z: z_new as f64
        };

        protocol.teleport(new_loc)
    }
}
