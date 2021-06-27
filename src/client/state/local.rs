/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/27/21, 3:15 PM
 */

use crate::client::pathfind::context::{MoveNode};
use crate::storage::block::BlockLocation;
use crate::client::physics::Physics;
use crate::client::state::inventory::Inventory;
use crate::protocol::ClientInfo;
use crate::client::follow::Follower;
use crate::types::Dimension;
use crate::client::pathfind::implementations::Problem;
use crate::client::pathfind::implementations::novehicle::TravelProblem;

type Prob = Box<dyn Problem<Node=MoveNode>>;

pub struct MineTask {
    pub ticks: usize,
    pub location: BlockLocation
}

pub struct LocalState {
    pub ticks: usize,
    pub mining: Option<MineTask>,
    pub follow_closest: bool,
    pub bot_id: u32,
    pub physics: Physics,
    pub disconnected: bool,
    pub inventory: Inventory,
    pub info: ClientInfo,
    pub alive: bool,
    pub dimension: Dimension,
    pub follower: Option<Follower>,

    pub travel_problem: Option<Prob>,

    /// so we can restart if we get an invalid result
    pub last_problem: Option<Prob>,
}

impl LocalState {

    pub fn travel_to_block(&mut self, goal: BlockLocation) {
        let from = self.physics.location().into();
        println!("starting nav");
        let problem = box TravelProblem::new(from, goal);

        self.travel_problem = Some(problem);
    }
}
