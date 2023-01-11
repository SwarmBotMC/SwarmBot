use interfaces::types::block_data::BlockData;

use crate::{
    client::{
        pathfind::context::PathConfig,
        state::global::{mine_alloc::MineAlloc, world_players::WorldPlayers},
    },
    storage::{blocks::WorldBlocks, entities::WorldEntities},
};

pub mod mine_alloc;
pub mod world_players;

/// The global state shared between bots
#[derive(Default)]
pub struct GlobalState {
    /// all the blocks in the world
    pub blocks: WorldBlocks,
    /// mine tasks
    pub mine: MineAlloc,

    /// all block data (which ID is which block)
    pub block_data: BlockData,

    /// all entities
    pub entities: WorldEntities,

    /// all players
    pub players: WorldPlayers,

    /// ticks since the first bot has joined
    pub ticks: usize,

    /// config for traveling
    pub travel_config: PathConfig,
}

impl GlobalState {
    pub fn init() -> Self {
        Self::default()
    }
}
