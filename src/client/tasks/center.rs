/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 2:06 AM
 */

use crate::client::tasks::{TaskTrait};
use crate::protocol::InterfaceOut;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;
use crate::storage::block::BlockLocation;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;

pub struct CenterTask;

impl TaskTrait for CenterTask {
    fn tick(&mut self, _out: &mut impl InterfaceOut, local: &mut LocalState, _global: &mut GlobalState) -> bool {

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
