use std::rc::Rc;

use tokio::sync::Notify;

use crate::client::pathfind::context::{MoveNode, Costs};
use crate::client::pathfind::incremental::AStar;
use crate::client::pathfind::progress_checker::{NoVehicleGoalCheck, NoVehicleHeuristic};
use crate::storage::block::BlockLocation;


use crate::client::state::travel::TravelProblem;
use crate::client::physics::Physics;
use crate::client::state::inventory::Inventory;
use crate::protocol::ClientInfo;
use crate::client::follow::Follower;
use crate::types::Dimension;

pub struct LocalState {
    pub ticks: usize,
    pub bot_id: u32,
    pub physics: Physics,
    pub disconnected: bool,
    pub inventory: Inventory,
    pub costs: Costs,
    pub info: ClientInfo,
    pub alive: bool,
    pub dimension: Dimension,
    pub follower: Option<Follower>,

    pub travel_problem: Option<TravelProblem>,

    // so we can restart if we get an invalid result
    pub last_problem: Option<TravelProblem>,
}

impl LocalState {

    pub fn travel_to_block(&mut self, goal: BlockLocation) {
        let from = self.physics.location().into();

        let start_node = MoveNode::simple(from);

        let problem = TravelProblem::new(start_node, goal);

        self.travel_problem = Some(problem);
        // notifier.notified().await;
    }
}
