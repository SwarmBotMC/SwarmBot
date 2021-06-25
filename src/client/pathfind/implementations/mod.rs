use std::collections::HashMap;

use crate::client::pathfind::context::{GlobalContext, MoveNode, MoveRecord};
use crate::client::pathfind::incremental::{AStar, Node, PathResult};
use crate::client::pathfind::moves::Movements;
use crate::client::pathfind::traits::{GoalCheck, Heuristic, Progression, Progressor};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::storage::block::{BlockLocation, BlockState};
use std::time::Instant;

pub mod novehicle;
pub mod build;


pub trait Problem: Send + Sync {
    type Node: Node;
    fn iterate_until(&mut self, time: Instant, local: &mut LocalState, global: &GlobalState) -> Increment<PathResult<<Self::Node as Node>::Record>>;
    fn recalc(&mut self, context: Self::Node);
}

type BlockLookup = HashMap<BlockLocation, BlockState>;

struct PlayerProblem<H: Heuristic<MoveNode>, G: GoalCheck<MoveNode>> {
    a_star: AStar<MoveNode>,
    heuristic: H,
    goal_checker: G,
    blocks_to_change: BlockLookup,
}


impl<H: Heuristic<MoveNode> + Send + Sync, G: GoalCheck<MoveNode> + Send + Sync> PlayerProblem<H, G> {
    pub fn new(start: MoveNode, heuristic: H, goal_checker: G, blocks_to_change: BlockLookup) -> PlayerProblem<H, G> {
        let a_star = AStar::new(start);
        Self {
            heuristic,
            blocks_to_change,
            a_star,
            goal_checker,
        }
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
            blocks_to_change: &self.blocks_to_change,
            path_config: &global.travel_config,
            world: &global.world_blocks,
        };
        let progressor = GenericProgressor { ctx };
        self.a_star.iterate_until(end_at, &self.heuristic, &progressor, &self.goal_checker)
    }

    fn recalc(&mut self, context: Self::Node) {
        self.a_star = AStar::new(context);
    }
}
