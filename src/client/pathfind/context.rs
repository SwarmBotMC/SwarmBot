/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::client::pathfind::incremental::Node;
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::blocks::WorldBlocks;

#[derive(Clone)]
pub struct Costs {
    pub block_walk: f64,
    pub mine_unrelated: f64,
    pub mine_required: f64,
    pub place_unrelated: f64,
    pub place_required: f64,
    pub ascend: f64,
    pub no_breathe_mult: f64,
    pub fall: f64,
}

pub struct PathConfig {
    pub costs: Costs,
    pub parkour: bool,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            costs: Costs {
                block_walk: 1.0,
                mine_unrelated: 20.0,
                ascend: 1.0,
                no_breathe_mult: 3.0,
                fall: 1.0,
                place_unrelated: 20.0,
                mine_required: 1.0,
                place_required: 1.0,
            },
            parkour: true,
        }
    }
}

#[derive(Clone)]
pub struct GlobalContext<'a> {
    pub blocks_to_change: &'a HashMap<BlockLocation, BlockState>,
    pub path_config: &'a PathConfig,
    pub world: &'a WorldBlocks,
}

#[derive(Debug)]
pub struct MoveNode {
    /// # Building
    /// Suppose we are building a structure. We will take the number
    /// of blocks which need to be changed and make blocks_needed_change that
    /// amount. Then each time we modify a block we can check if we are changing
    /// a block that needs to be changed. If we are changing to the right block,
    /// we decrement the value. If we set it to the wrong block, we increment.
    /// A goal can only be reached when blocks_needed_change is 0
    /// # Mining
    /// Suppose we are mining an area. Then this will be the number of blocks
    /// we want to be air.
    pub blocks_needed_change: usize,

    /// The current location of the user
    pub location: BlockLocation,

    /// All the modified blocks we currently have that are different than the global state

    /// The action needed to obtain this node. Note: This different actions do not mean this node is not equal
    pub action_to_obtain: Option<Action>,

    /// The number of 'throwaway' blocks we have, i.e., for bridging
    pub throwaway_block_count: usize,
}

impl MoveNode {
    pub fn simple(location: BlockLocation) -> MoveNode {
        MoveNode {
            blocks_needed_change: 0,
            location,
            action_to_obtain: None,
            throwaway_block_count: 0,
        }
    }

    pub fn new(location: BlockLocation, blocks_to_change: &std::collections::HashMap<BlockLocation, BlockState>) -> MoveNode {
        let blocks_needed_change = blocks_to_change.len();

        MoveNode {
            blocks_needed_change,
            location,
            action_to_obtain: None,
            throwaway_block_count: 0,
        }
    }

    pub fn from(previous: &MoveNode) -> MoveNode {
        let mut previous = previous.clone();
        previous.action_to_obtain = None;
        previous
    }
}

impl Clone for MoveNode {
    fn clone(&self) -> Self {
        Self {
            blocks_needed_change: self.blocks_needed_change,
            location: self.location,
            action_to_obtain: None,
            throwaway_block_count: self.throwaway_block_count,
        }
    }
}


impl Node for MoveNode {
    type Record = MoveRecord;

    fn get_record(&self) -> Self::Record {
        let &MoveNode { location, throwaway_block_count, action_to_obtain, .. } = self;

        let state = MoveState {
            location,
            throwaway_block_count,
        };

        Self::Record {
            state,
            action_to_obtain,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Action {
    Change(BlockLocation, BlockState),
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct MoveState {
    pub location: BlockLocation,
    pub throwaway_block_count: usize,
}

#[derive(Clone)]
pub struct MoveRecord {
    pub state: MoveState,
    pub action_to_obtain: Option<Action>,
}

impl PartialEq for MoveRecord {
    fn eq(&self, other: &Self) -> bool {
        self.state.eq(&other.state)
    }
}

impl Eq for MoveRecord {}

impl Hash for MoveRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state);
    }
}
