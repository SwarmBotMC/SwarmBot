/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::hash::{Hash, Hasher};

use crate::client::pathfind::incremental::Node;
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::blocks::WorldBlocks;

#[derive(Clone)]
pub struct Costs {
    pub block_walk: f64,
    pub block_parkour: f64,
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
                block_parkour: 1.5,
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
    pub path_config: &'a PathConfig,
    pub world: &'a WorldBlocks,
}

#[derive(Debug)]
pub struct MoveNode {
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
            location,
            action_to_obtain: None,
            throwaway_block_count: 0,
        }
    }

    pub fn new(location: BlockLocation) -> MoveNode {

        MoveNode {
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

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct MoveState {
    pub location: BlockLocation,
    pub throwaway_block_count: usize,
}

#[derive(Clone, Debug)]
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
