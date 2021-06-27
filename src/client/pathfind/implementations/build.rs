/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/27/21, 3:15 PM
 */

use crate::storage::block::{BlockLocation, BlockState};
use std::collections::{HashMap, HashSet};
use crate::client::pathfind::traits::{Heuristic, GoalCheck};
use crate::client::pathfind::context::MoveNode;
use crate::client::pathfind::implementations::{PlayerProblem, Problem};


struct HeuristicImpl;

impl Heuristic<MoveNode> for HeuristicImpl {
    fn heuristic(&self, input: &MoveNode) -> f64 {
        input.blocks_needed_change as f64
    }
}


struct GoalCheckImpl;

impl GoalCheck<MoveNode> for GoalCheckImpl {
    fn is_goal(&self, input: &MoveNode) -> bool {
        input.blocks_needed_change == 0
    }
}

pub struct BuildNoVehicleProblem;

impl BuildNoVehicleProblem {
    pub fn change(start: BlockLocation, blocks_to_change: HashMap<BlockLocation, BlockState>) -> impl Problem<Node=MoveNode>{

        let start = MoveNode::new(start, &blocks_to_change);

        return PlayerProblem::new(start, HeuristicImpl, GoalCheckImpl, blocks_to_change)
    }

    pub fn mine(start: BlockLocation, blocks: HashSet<BlockLocation>) -> impl Problem<Node=MoveNode>{
        let change = blocks.iter().map(|&loc| (loc, BlockState::AIR)).collect();
        return BuildNoVehicleProblem::change(start, change)
    }
}
