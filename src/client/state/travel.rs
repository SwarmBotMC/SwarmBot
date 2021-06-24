use crate::client::pathfind::context::{MoveNode, MoveRecord};
use crate::client::pathfind::incremental::{AStar, PathResult};
use crate::client::pathfind::progress_checker::{NoVehicleGoalCheck, NoVehicleHeuristic, Progressor};
use crate::storage::block::BlockLocation;
use crate::client::timing::Increment;
use std::time::Duration;

pub struct TravelPath {
    blocks: Vec<BlockLocation>,
}

pub struct TravelProblem {
    a_star: AStar<MoveNode>,
    heuristic: NoVehicleHeuristic,
    goal_checker: NoVehicleGoalCheck,
}

impl TravelProblem {
    pub fn new(start: MoveNode, goal: BlockLocation) -> TravelProblem {
        let heuristic = NoVehicleHeuristic {
            move_cost: 1.0,
            goal,
        };

        let a_star = AStar::new(start);
        let goal_checker = NoVehicleGoalCheck::new(goal);
        Self {
            heuristic,
            a_star,
            goal_checker,
        }
    }
}

impl TravelProblem {
    pub fn iterate_for(&mut self, duration: Duration, progressor: &impl Progressor<MoveNode>) -> Increment<PathResult<MoveRecord>> {
        self.a_star.iterate_for(duration, &self.heuristic, progressor, &self.goal_checker)
    }

    pub fn recalc(&mut self, context: MoveNode) {
        self.a_star = AStar::new(context);
    }
}
