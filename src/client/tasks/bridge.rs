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
        pathfind::moves::CardinalDirection,
        physics::{speed::Speed, Line},
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::{Face, InterfaceOut},
    storage::block::BlockLocation,
    types::{Direction, Displacement},
};

pub struct BridgeTask {
    count: u32,
    place_against: BlockLocation,
    direction: CardinalDirection,
}

impl BridgeTask {
    pub fn new(count: u32, direction: CardinalDirection, local: &LocalState) -> BridgeTask {
        let start = BlockLocation::from(local.physics.location()).below();
        Self {
            count,
            place_against: start,
            direction,
        }
    }
}

impl TaskTrait for BridgeTask {
    fn tick(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        _global: &mut GlobalState,
    ) -> bool {
        let displacement = Displacement::from(self.direction.unit_change());

        let direction = Direction::from(-displacement);

        local.physics.look(direction);
        local.physics.line(Line::Backward);
        local.physics.speed(Speed::WALK);

        let target_loc = self.place_against.true_center();
        let current_loc = local.physics.location();

        let place = match self.direction {
            CardinalDirection::North => target_loc.x - current_loc.x < (-0.6),
            CardinalDirection::South => target_loc.x - current_loc.x > (-0.4 + 0.5),
            CardinalDirection::West => target_loc.z - current_loc.z > (0.4 - 0.5),
            CardinalDirection::East => target_loc.z + current_loc.z > (0.4 + 0.5),
        };

        if place {
            let face = Face::from(self.direction);
            local.physics.place_hand_face(self.place_against, face);
            let change = BlockLocation::from(self.direction.unit_change());
            self.place_against = self.place_against + change;
            self.count -= 1;
        }

        self.count == 0
    }
}
