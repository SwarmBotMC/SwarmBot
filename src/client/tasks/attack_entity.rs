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

use crate::client::pathfind::implementations::novehicle::TravelProblem;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::hit_entity::HitEntityTask;
use crate::client::tasks::navigate::NavigateProblem;
use crate::client::tasks::stream::TaskStream;
use crate::client::tasks::Task;
use crate::protocol::InterfaceOut;
use crate::types::Location;

pub struct AttackEntity {
    id: u32,
}

impl AttackEntity {
    pub fn new(id: u32) -> Self {
        Self {id}
    }
}

impl TaskStream for AttackEntity {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task> {
        let current_location = local.physics.location();

        // we cannot do anything if we do not know the location so we end the task
        let entity_location = global.entities.by_id(self.id)?.location;

        let dist2 = entity_location.dist2(current_location);

        const THRESHOLD_DIST: f64 = 3.0;

        if dist2 < THRESHOLD_DIST * THRESHOLD_DIST {  // we can hit the entity
            let hit = HitEntityTask::new(self.id);
            Some(hit.into())
        } else { // we need to travel to them
            let travel = TravelProblem::navigate_block(current_location.into(), entity_location.into());
            let task = NavigateProblem::from(travel);

            Some(task.into())
        }
    }
}
