/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:18 AM
 */

use crate::client::tasks::lazy_stream::LazyStream;
use std::collections::VecDeque;
use crate::storage::block::BlockLocation;
use crate::client::state::global::GlobalState;
use float_ord::FloatOrd;
use crate::client::tasks::stream::TaskStream;
use crate::protocol::InterfaceOut;
use crate::client::state::local::LocalState;
use crate::client::tasks::Task;
use crate::client::tasks::mine::MineTask;
use itertools::Itertools;

pub type MineLayerTask = LazyStream<MineLayer>;

pub struct MineLayer {
    blocks_to_mine: VecDeque<BlockLocation>,
    start_loc: BlockLocation,
}

impl MineLayer {
    pub fn new(origin_loc: BlockLocation, global: &GlobalState) -> Option<MineLayer> {
        const RADIUS: u8 = 3;

        let blocks_to_mine = global.blocks
            .y_slice(origin_loc, RADIUS, |state| state.kind().mineable(&global.block_data))?
            .into_iter()
            .sorted_unstable_by_key(|&loc| {
                let priority = if loc == origin_loc {
                    // we always want to do our current loc last so we don't fall when mining other blocks
                    f64::INFINITY
                } else {
                    // else sort closest to furthes away
                    loc.dist2(origin_loc)
                };
                FloatOrd(priority)
            })
            .collect();


        Some(Self {
            blocks_to_mine,
            start_loc: origin_loc,
        })
    }
}

impl TaskStream for MineLayer {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task> {
        self.blocks_to_mine
            .pop_front()
            .map(|loc| MineTask::new(loc, out, local, global).into())
    }
}
