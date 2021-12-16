// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    client::{physics::Physics, state::local::inventory::PlayerInventory},
    protocol::{ClientInfo, Face},
    storage::block::BlockLocation,
    types::Dimension,
};

pub mod inventory;

pub enum TaskKind {
    Mine(BlockLocation, Face),
    Eat,
}

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
        Self::new(
            0,
            ClientInfo {
                username: "abc".to_string(),
                uuid: Default::default(),
                entity_id: 0,
            },
        )
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
