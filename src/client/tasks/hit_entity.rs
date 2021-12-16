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
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
    types::Displacement,
};

pub struct HitEntityTask {
    id: u32,
}

impl HitEntityTask {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl TaskTrait for HitEntityTask {
    fn tick(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        let mut action = || {
            let entity_loc = global.entities.by_id(self.id)?.location;

            // hit the head
            let target_loc = entity_loc + Displacement::EYE_HEIGHT;

            local.physics.look_at(target_loc);
            out.look(local.physics.direction());
            out.attack_entity(self.id);
            out.swing_arm();

            Some(())
        };

        action();
        true
    }
}
