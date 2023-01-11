use crate::{
    client::{physics::Physics, state::local::inventory::PlayerInventory},
    protocol::ClientInfo,
    types::Dimension,
};

pub mod inventory;

/// The local state which only each bot knows
///
/// - this can be mutably modified by each bot at any time because it is the
///   only one who
/// owns the [`LocalState`]
pub struct LocalState {
    pub ticks: usize,
    pub health: f32,
    pub food: u8,
    pub follow_closest: bool,
    pub bot_id: u32,
    pub physics: Physics,
    pub disconnected: bool,
    pub inventory: PlayerInventory,
    pub info: ClientInfo,
    pub alive: bool,
    pub dimension: Dimension,
}

impl LocalState {
    #[allow(unused)]
    pub fn mock() -> Self {
        Self::new(
            0,
            ClientInfo {
                username: "abc".to_string(),
                uuid: Default::default(),
                entity_id: 0,
            },
        )
    }
}

impl LocalState {
    pub fn new(bot_id: u32, info: ClientInfo) -> Self {
        Self {
            ticks: 0,
            health: 0.0,
            food: 0,
            follow_closest: false,
            bot_id,
            physics: Physics::default(),
            disconnected: false,
            inventory: PlayerInventory::default(),
            alive: true,
            dimension: Dimension::Overworld,
            info,
        }
    }
}
