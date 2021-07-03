/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::client::physics::Physics;
use crate::client::state::inventory::PlayerInventory;
use crate::protocol::{ClientInfo, Face};
use crate::storage::block::BlockLocation;
use crate::types::Dimension;


pub enum TaskKind { Mine(BlockLocation, Face), Eat }

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
    pub fn mock() -> LocalState {
        Self::new(0, ClientInfo {
            username: "abc".to_string(),
            uuid: Default::default(),
            entity_id: 0,
        })
    }

    pub fn new(bot_id: u32, info: ClientInfo) -> LocalState {
        LocalState {
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
