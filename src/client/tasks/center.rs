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
        physics::{speed::Speed, Line},
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
};
use interfaces::types::BlockLocation;

pub struct CenterTask;

impl TaskTrait for CenterTask {
    fn tick(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        _global: &mut GlobalState,
    ) -> bool {
        const THR: f64 = 0.05;
        const VEL_THR: f64 = 0.05;
        let loc = local.physics.location();
        let vel = local.physics.velocity();
        let mut center_loc = BlockLocation::from(loc).true_center();
        center_loc.y = loc.y;

        if loc.dist2(center_loc) < THR * THR && vel.mag2() < VEL_THR * VEL_THR {
            return true;
        }

        local.physics.look_at(center_loc);
        local.physics.line(Line::Forward);
        local.physics.speed(Speed::SNEAK);

        false
    }
}
