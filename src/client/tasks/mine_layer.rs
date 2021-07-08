/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:18 AM
 */

use float_ord::FloatOrd;


use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::lazy_stream::LazyStream;
use crate::client::tasks::mine::MineTask;
use crate::client::tasks::stream::TaskStream;
use crate::client::tasks::Task;
use crate::protocol::InterfaceOut;
use crate::storage::block::BlockLocation;

pub type MineLayerTask = LazyStream<MineLayer>;

pub struct MineLayer;

impl TaskStream for MineLayer {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task> {
        const RADIUS: u8 = 3;

        let origin_loc = BlockLocation::from(local.physics.location()).below();

        let block_to_mine = global.blocks
            .y_slice(origin_loc, RADIUS, |state| state.kind().mineable(&global.block_data))?
            .into_iter()
            .min_by_key(|&loc| {
                let priority = if loc == origin_loc {
                    // we always want to do our current loc last so we don't fall when mining other blocks
                    f64::INFINITY
                } else {
                    // else sort closest to furthes away
                    loc.dist2(origin_loc)
                };
                FloatOrd(priority)
            })?;

        Some(MineTask::new(block_to_mine, out, local, global).into())
    }
}
