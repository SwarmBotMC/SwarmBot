use crate::client::state::inventory::Inventory;
use crate::client::pathfind::context::{Costs, MoveContext};
use crate::client::instance::{ClientInfo, TravelProblem};
use crate::storage::block::BlockLocation;
use crate::client::state::Dimension;
use crate::client::follow::Follower;
use crate::types::Location;
use crate::client::pathfind::incremental::AStar;
use tokio::sync::Notify;
use std::rc::Rc;
use crate::client::pathfind::progress_checker::{NoVehicleHeuristic, NoVehicleGoalCheck};

pub struct State {
    pub ticks: usize,
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

impl State {
    pub fn block_location(&self) -> BlockLocation {
        let Location { x, y, z } = self.location;
        BlockLocation(x.floor() as i64, y.floor() as i64, z.floor() as i64)
    }

    async fn travel_to_block(&mut self, goal: BlockLocation) {
        let from = self.block_location();


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
            notifier: notifier.clone(),
        };


        self.travel_problem = Some(problem);

        notifier.notified().await;
    }
}
