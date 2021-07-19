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

use std::collections::HashMap;
use std::time::Instant;

use crate::client::pathfind::context::{GlobalContext, MoveNode, MoveRecord};
use crate::client::pathfind::incremental::{AStar, Node, PathResult};
use crate::client::pathfind::moves::Movements;
use crate::client::pathfind::traits::{GoalCheck, Heuristic, Progression, Progressor};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::storage::block::{BlockLocation, BlockState};

pub mod novehicle;


pub trait Problem: Send + Sync {
    type Node: Node;
    fn iterate_until(&mut self, time: Instant, local: &mut LocalState, global: &GlobalState) -> Increment<PathResult<<Self::Node as Node>::Record>>;
    fn recalc(&mut self, context: Self::Node);
}

type BlockLookup = HashMap<BlockLocation, BlockState>;

pub struct PlayerProblem<H: Heuristic<MoveNode>, G: GoalCheck<MoveNode>> {
    a_star: AStar<MoveNode>,
    heuristic: H,
    goal_checker: G,
}


impl<H: Heuristic<MoveNode> + Send + Sync, G: GoalCheck<MoveNode> + Send + Sync> PlayerProblem<H, G> {
    pub fn new(start: MoveNode, heuristic: H, goal_checker: G) -> PlayerProblem<H, G> {
        let a_star = AStar::new(start);
        Self {
            heuristic,
            a_star,
            goal_checker,
        }
    }

    pub fn set_max_millis(&mut self, value: u128) {
        self.a_star.set_max_millis(value);
    }
}


#[derive(Clone)]
struct GenericProgressor<'a> {
    ctx: GlobalContext<'a>,
}

impl Progressor<MoveNode> for GenericProgressor<'_> {
    fn progressions(&self, location: &MoveNode) -> Progression<MoveNode> {
        Movements::obtain_all(location, &self.ctx)
    }
}

impl<H: Heuristic<MoveNode> + Send + Sync, G: GoalCheck<MoveNode> + Send + Sync> Problem for PlayerProblem<H, G> {
    type Node = MoveNode;

    fn iterate_until(&mut self, end_at: Instant, _: &mut LocalState, global: &GlobalState) -> Increment<PathResult<MoveRecord>> {
        let ctx = GlobalContext {
            path_config: &global.travel_config,
            world: &global.blocks,
        };
        let progressor = GenericProgressor { ctx };
        self.a_star.iterate_until(end_at, &self.heuristic, &progressor, &self.goal_checker)
    }

    fn recalc(&mut self, context: Self::Node) {
        self.a_star = AStar::new(context);
    }
}
