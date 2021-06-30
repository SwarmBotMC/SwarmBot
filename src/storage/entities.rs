/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use indexmap::map::IndexMap;

use crate::types::{Location, LocationOrigin};

#[derive(Default)]
pub struct EntityData {
    pub location: Location,
    pub owner: Option<u32>,
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

    pub fn put_entity(&mut self, entity_id: u32, bot_id: u32, location: Location) {
        self.entities.insert(entity_id, EntityData {
            location,
            owner: Some(bot_id),
        });
    }
}
