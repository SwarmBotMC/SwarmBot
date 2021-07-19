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

use indexmap::map::IndexMap;

use crate::types::{Location, LocationOrigin};

pub enum EntityKind {
    Normal,
    Player {
        uuid: u128,
    }
}

pub struct EntityData {
    pub location: Location,
    pub owner: Option<u32>,
    pub kind: EntityKind
}

#[derive(Default)]
pub struct WorldEntities {
    /// map of Id => EntityData
    entities: IndexMap<u32, EntityData>,
}

impl WorldEntities {
    pub fn update_entity(&mut self, entity_id: u32, bot_id: u32, location: LocationOrigin) {
        let entity = match self.entities.get_mut(&entity_id) {
            None => return, // probably an unimportant entity (i.e., TNT)
            Some(entity) => entity
        };
        let id = entity.owner.get_or_insert(bot_id);
        if *id == bot_id {
            entity.location.apply_change(location);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(&u32, &EntityData)> + '_ {
        self.entities.iter()
    }

    pub fn by_id(&self, id: u32) -> Option<&EntityData> {
        self.entities.get(&id)
    }

    pub fn by_player_uuid(&self, uuid: u128) -> Option<u32> {
        self.iter().find_map(|(id, data)| match data.kind {
            EntityKind::Normal => None,
            EntityKind::Player{ uuid: player_uuid } => (player_uuid == uuid).then_some(*id)
        })
    }

    pub fn remove_entity(&mut self, entity_id: u32, bot_id: u32) {
        let entity = self.entities.get_mut(&entity_id);
        let entity = match entity {
            None => return,
            Some(inner) => inner
        };

        if entity.owner == Some(bot_id) {
            entity.owner = None;
        }
    }

    pub fn put_entity(&mut self, entity_id: u32, bot_id: u32, location: Location, kind: EntityKind) {
        self.entities.insert(entity_id, EntityData {
            location,
            owner: Some(bot_id),
            kind
        });
    }
}
