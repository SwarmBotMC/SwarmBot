/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::bootstrap::block_data::BlockData;
use crate::client::pathfind::context::PathConfig;
use crate::client::state::global::mine_alloc::MineAlloc;
use crate::storage::blocks::WorldBlocks;
use crate::storage::entities::WorldEntities;
use crate::client::state::global::world_players::WorldPlayers;

pub mod mine_alloc;
pub mod world_players;

#[derive(Default)]
pub struct GlobalState {
    pub blocks: WorldBlocks,
    pub mine: MineAlloc,
    pub block_data: BlockData,
    pub entities: WorldEntities,
    pub players: WorldPlayers,
    pub ticks: usize,
    pub travel_config: PathConfig,
}

impl GlobalState {
    pub fn init() -> GlobalState {
        GlobalState::default()
    }

    /// # Goal
    /// we want to assign regions to explore for each bot
    /// we want to explore in rings
    ///
    /// ```
    /// 33333
    /// 32223
    /// 32123
    /// 32223
    /// 33333
    /// ```
    ///
    /// Think of this as a priority queue where low numbers have a higher priority. It is easy to connect all regions
    /// because the chunks loaded are in a square in Minecraft and not a circle. Therefore, we can make sure sections
    /// will not collide.
    ///
    /// # Assignment
    /// ## Initial
    /// Initially, this will just be a priority queue. All bots will get assigned a slot and walk to it
    ///
    /// ## Next Step
    /// A naïve approach would be always taking the region with the least priority and breaking ties with distance.
    /// However, assume a bot is at an x and the last remaining region at the tie-breaking priority is an o
    /// ```
    /// ..x
    /// ...
    /// o..
    /// ```
    ///
    /// This would be a long traversal. In addition, assume  this was a thousand blocks away. This would take a lot of extra time. Ideally we would
    /// have a bot that will finish the task in a little period of time go to it. Instead we will have bots choose the smallest priority adjacent to
    /// it else if there are no adj the closest next smallest. Let's see how this would play out
    ///
    ///```
    /// 4321.
    /// 5..0.
    /// 6....
    /// 78...
    /// 9....
    ///```
    /// or equally likely
    ///
    ///```
    /// ...12
    /// ...03
    /// 3...4
    /// 12..5
    /// 09876
    ///```
    /// ## Data structure
    /// We want to make it easy for bots to follow the graph. Let us denote each grid as `(x,y)`, where the priority is `max(abs(x),abs(y))`
    ///
    ///```
    ///(-1, 1)(0, 1)(1, 1)
    ///(-1, 0)(0, 0)(1, 0)
    ///(-1,-1)(0,-1)(1,-1)
    ///```
    ///
    /// We _could_ use a [`std::collections::hash::HashSet`] with an i32 tuple, but we could also use a wrapping structure
    ///
    /// ```
    /// 123
    /// 804
    /// 765
    /// ```
    ///
    /// We will use a HashMap for now though since it is simpler
    /// the lengths are 1, (3*3 - prev) = 8, (5*5) - prev = 17.
    /// There is a clock-wise wrapping where the top left is the first element.
    ///
    async fn explore_circular(&mut self) {
        todo!()
        // // initial

        // let mut left_over = HashSet::new();
        // left_over.insert((0, 0));
        // let mut r = 0;
    }
}
