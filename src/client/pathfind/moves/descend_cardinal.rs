use crate::pathfind::moves::{Move, MoveResult};
use crate::pathfind::context::Context;
use crate::pathfind::BlockLocation;

struct DescendCardinal;

impl Move for DescendCardinal {
    fn on_move(&self, from: BlockLocation, context: &Context) -> MoveResult {
        todo!()
    }
}
