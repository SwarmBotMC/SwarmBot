use float_ord::FloatOrd;
use interfaces::types::BlockLocation;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{lazy_stream::LazyStream, mine::MineTask, stream::TaskStream, Task},
    },
    protocol::InterfaceOut,
};

pub type MineLayerTask = LazyStream<MineLayer>;

pub struct MineLayer;

impl TaskStream for MineLayer {
    fn poll(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> Option<Task> {
        const RADIUS: u8 = 3;

        let origin_loc = BlockLocation::from(local.physics.location()).below();

        let block_to_mine = global
            .blocks
            .y_slice(origin_loc, RADIUS, |state| {
                state.kind().mineable(&global.block_data)
            })?
            .into_iter()
            .min_by_key(|&loc| {
                let priority = if loc == origin_loc {
                    // we always want to do our current loc last so we don't fall when mining other
                    // blocks
                    f64::INFINITY
                } else {
                    // else sort closest to furthest away
                    loc.dist2(origin_loc)
                };
                FloatOrd(priority)
            })?;

        Some(MineTask::new(block_to_mine, out, local, global).into())
    }
}
