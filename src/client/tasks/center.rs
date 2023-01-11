use interfaces::types::BlockLocation;

use crate::{
    client::{
        physics::{speed::Speed, Line},
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
    },
    protocol::InterfaceOut,
};

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
