/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::client::follow::Follower;
use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::novehicle::TravelProblem;
use crate::client::pathfind::implementations::Problem;
use crate::client::physics::Physics;
use crate::client::state::inventory::Inventory;
use crate::protocol::ClientInfo;
use crate::storage::block::BlockLocation;
use crate::types::{Dimension, Location};

type Prob = Box<dyn Problem<Node=MoveNode>>;

pub enum TaskKind {Mine(BlockLocation), Eat}

pub struct Task {
   pub ticks: usize,
   pub kind: TaskKind
}

pub struct LocalState {
    pub ticks: usize,
    pub health: f32,
    pub food: u8,
    pub task: Option<Task>,
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

    pub fn mock() -> LocalState {
        Self::new(0, ClientInfo {
            username: "abc".to_string(),
            uuid: Default::default(),
            entity_id: 0
        })
    }

    pub fn new(bot_id: u32, info: ClientInfo) -> LocalState {
        LocalState {
            ticks: 0,
            health: 0.0,
            food: 0,
            task: None,
            follow_closest: false,
            bot_id,
            physics: Physics::default(),
            disconnected: false,
            inventory: Inventory {},
            alive: true,
            dimension: Dimension::Overworld,
            follower: None,
            info,
            travel_problem: None,
            last_problem: None,
        }
    }
    pub fn travel_to_block(&mut self, goal: BlockLocation) {
        let from = self.physics.location().into();
        println!("starting nav");
        let problem = box TravelProblem::new(from, goal);

        self.travel_problem = Some(problem);
    }
}
