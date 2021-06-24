use crate::storage::blocks::WorldBlocks;
use crate::storage::block::{BlockLocation, BlockState, BlockApprox};
use crate::client::pathfind::incremental::Node;
use std::hash::{Hash, Hasher};
use fasthash::{FastHasher};

#[derive(Clone)]
pub struct Costs {
    pub block_walk: f64,
    pub ascend: f64,
    pub fall: f64,
    pub block_place: f64
}

pub struct PathConfig {
    pub costs: Costs,
    pub parkour: bool
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            costs: Costs {
                block_walk: 1.0,
                ascend: 1.0,
                fall: 1.0,
                block_place: 200.0
            },
            parkour: true
        }
    }
}

#[derive(Clone)]
pub struct GlobalContext<'a> {
    pub path_config: &'a PathConfig,
    pub world: &'a WorldBlocks,
}

#[derive(Clone, Debug)]
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
    modified_blocks: im_rc::HashMap<BlockLocation, BlockState>,

    pub current_hash: u64,

    /// The action needed to obtain this node. Note: This different actions do not mean this node is not equal
    pub action_to_obtain: Option<Action>,

    /// The number of 'throwaway' blocks we have, i.e., for bridging
    pub throwaway_block_count: usize
}

impl MoveNode {

    pub fn simple(location: BlockLocation) -> MoveNode {
        MoveNode {
            blocks_needed_change: 0,
            location,
            modified_blocks: Default::default(),
            current_hash: 0,
            action_to_obtain: None,
            throwaway_block_count: 0
        }
    }

    pub fn from(previous: &MoveNode) -> MoveNode {
        previous.clone()
    }

    pub fn set_block(&mut self, location: BlockLocation, state: BlockState){
        self.modified_blocks.insert(location, state);

        let mut hasher = fasthash::MetroHasher::new();
        location.hash(&mut hasher);
        state.hash(&mut hasher);

        // Order invariant hash. Unfortunately if a block is added and then removed this will be broken but oh well
        self.current_hash ^= hasher.finish();
    }

    pub fn get_block(&self, location: BlockLocation, blocks: &WorldBlocks) -> Option<BlockApprox> {
        match self.modified_blocks.get(&location) {
            None => blocks.get_block(location),
            Some(local) => Some(BlockApprox::Realized(*local))
        }
    }

}


impl Node for MoveNode {
    type Record = MoveRecord;

    fn get_record(&self) -> Self::Record {

        let &MoveNode  {current_hash, location, throwaway_block_count, action_to_obtain, ..} = self;

        let state = MoveState {
            location,
            current_hash,
            throwaway_block_count
        };

        Self::Record {
            state,
            action_to_obtain
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Action {
    Mine(BlockLocation),
    Place(BlockLocation, BlockState)
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct MoveState {
    pub location: BlockLocation,
    current_hash: u64,
    pub throwaway_block_count: usize
}

#[derive(Clone)]
pub struct MoveRecord {
    pub state: MoveState,
    pub action_to_obtain: Option<Action>
}

impl PartialEq for MoveRecord {
    fn eq(&self, other: &Self) -> bool {
        self.state.eq(&other.state)
    }
}

impl Eq for MoveRecord{}

impl Hash for MoveRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state);
    }
}
