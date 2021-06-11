use crate::pathfind::world_blocks::WorldBlocks;

pub struct Costs {
    pub block_walk: f64,
    pub block_place: f64
}

pub struct Context {
    pub parkour: bool,
    pub costs: Costs,
    pub world: &'_ WorldBlocks,
}
