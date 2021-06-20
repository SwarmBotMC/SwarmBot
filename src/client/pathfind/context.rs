use crate::storage::world::WorldBlocks;
use crate::storage::block::BlockLocation;

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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MoveContext {
    pub location: BlockLocation,
    pub blocks_can_place: usize
}
