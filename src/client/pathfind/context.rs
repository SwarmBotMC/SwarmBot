use crate::storage::world::WorldBlocks;

pub struct Costs {
    pub block_walk: f64,
    pub block_place: f64
}

pub struct Context<'a> {
    pub parkour: bool,
    pub costs: Costs,
    pub world: &'a WorldBlocks,
}
