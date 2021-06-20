use std::rc::Rc;

use tokio::sync::Notify;

use crate::client::follow::Follower;
use crate::client::pathfind::context::{Costs, MoveContext};
use crate::client::pathfind::incremental::AStar;
use crate::client::pathfind::progress_checker::{NoVehicleGoalCheck, NoVehicleHeuristic};
use crate::client::state::Dimension;
use crate::client::state::inventory::Inventory;
use crate::storage::block::BlockLocation;
use crate::types::{Location};


use crate::protocol::ClientInfo;
use crate::client::state::travel::TravelProblem;

pub struct LocalState {
    pub ticks: usize,
    pub disconnected: bool,
    pub inventory: Inventory,
    pub costs: Costs,
    pub info: ClientInfo,
    pub destination: BlockLocation,
    pub alive: bool,
    pub dimension: Dimension,
    pub follower: Option<Follower>,
    pub travel_problem: Option<TravelProblem>,
    pub location: Location,
}

impl LocalState {

    pub fn travel_to_block(&mut self, goal: BlockLocation) {
        let from = self.location.into();


        // https://github.com/tokio-rs/tokio/releases/tag/tokio-0.2.12
        let notifier = Rc::new(Notify::new());

        let start_node = MoveContext {
            location: from,
            blocks_can_place: self.inventory.placeable_blocks(),
        };

        let heuristic = NoVehicleHeuristic {
            move_cost: self.costs.block_walk,
            goal,
        };

        let goal_checker = NoVehicleGoalCheck::new(goal);

        let a_star = AStar::new(start_node);


        let problem = TravelProblem {
            a_star,
            heuristic,
            goal_checker,
            notifier,
        };


        self.travel_problem = Some(problem);

        // notifier.notified().await;
    }
}
