/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::collections::HashMap;

use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::{PlayerProblem, Problem};
use crate::client::pathfind::traits::{GoalCheck, Heuristic};
use crate::storage::block::BlockLocation;
use crate::storage::blocks::ChunkLocation;

pub struct BlockGoalCheck {
    goal: BlockLocation,
}

pub struct ChunkGoalCheck {
    goal: ChunkLocation,
}

impl GoalCheck for ChunkGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        let cx = input.location.x >> 4;
        let cz = input.location.z >> 4;
        let chunk_loc = ChunkLocation(cx, cz);
        chunk_loc == self.goal
    }
}

impl BlockGoalCheck {
    pub fn new(goal: BlockLocation) -> Self {
        Self {
            goal
        }
    }
}

impl GoalCheck for BlockGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        let close_y = (input.location.y - self.goal.y).abs() <= 1;
        close_y && input.location.x == self.goal.x && input.location.z == self.goal.z
    }
}


pub struct BlockHeuristic {
    pub move_cost: f64,
    pub goal: BlockLocation,
}

impl Heuristic for BlockHeuristic {
    fn heuristic(&self, input: &MoveNode) -> f64 {
        let current = input.location;
        current.dist(self.goal) * self.move_cost * 0.2
    }
}

#[derive(Debug)]
pub struct ChunkHeuristic {
    pub move_cost: f64,
    pub center_x: i32,
    pub center_z: i32,
}

impl ChunkHeuristic {
    fn new(goal: ChunkLocation, move_cost: f64) -> Self {
        Self {
            move_cost,

            // the center of the chunk
            center_x: goal.0 << 4 + 8,
            center_z: goal.1 << 4 + 8,
        }
    }
}

impl Heuristic for ChunkHeuristic {
    fn heuristic(&self, input: &MoveNode) -> f64 {
        let dx = (input.location.x - self.center_x) as f64;
        let dz = (input.location.z - self.center_z) as f64;
        let dist2 = dx * dx + dz * dz;
        dist2.sqrt() * self.move_cost * 0.2
    }
}

pub struct TravelProblem;

pub type TravelBlockProblem = PlayerProblem<BlockHeuristic, BlockGoalCheck>;
pub type TravelChunkProblem = PlayerProblem<ChunkHeuristic, ChunkGoalCheck>;

impl TravelProblem {
    pub fn navigate_block(start: BlockLocation, goal: BlockLocation) -> TravelBlockProblem {
        let heuristic = BlockHeuristic { move_cost: 1.0, goal };
        let start_node = MoveNode::simple(start);
        let goal_checker = BlockGoalCheck::new(goal);
        PlayerProblem::new(start_node, heuristic, goal_checker, HashMap::new())
    }

    pub fn navigate_chunk(start: BlockLocation, goal: ChunkLocation) -> TravelChunkProblem {
        let heuristic = ChunkHeuristic::new(goal, 1.0);
        let start_node = MoveNode::simple(start);
        let goal_checker = ChunkGoalCheck { goal };
        PlayerProblem::new(start_node, heuristic, goal_checker, HashMap::new())
    }
}
