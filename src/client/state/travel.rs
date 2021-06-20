use crate::storage::block::BlockLocation;
use crate::client::pathfind::incremental::AStar;
use crate::client::pathfind::context::MoveContext;
use crate::client::pathfind::progress_checker::{NoVehicleHeuristic, NoVehicleGoalCheck};
use tokio::sync::Notify;
use std::rc::Rc;

pub struct TravelPath {
    blocks: Vec<BlockLocation>,
}

pub struct TravelProblem {
    pub a_star: AStar<MoveContext>,
    pub heuristic: NoVehicleHeuristic,
    pub goal_checker: NoVehicleGoalCheck,
    pub notifier: Rc<Notify>,
}

impl Drop for TravelProblem {
    fn drop(&mut self) {
        self.notifier.notify_one();
    }
}
