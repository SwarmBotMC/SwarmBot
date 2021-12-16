// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use float_ord::FloatOrd;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{lazy_stream::LazyStream, mine::MineTask, stream::TaskStream, Task},
    },
    protocol::InterfaceOut,
    storage::block::BlockLocation,
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
                    // else sort closest to furthes away
                    loc.dist2(origin_loc)
                };
                FloatOrd(priority)
            })?;

        Some(MineTask::new(block_to_mine, out, local, global).into())
    }
}
