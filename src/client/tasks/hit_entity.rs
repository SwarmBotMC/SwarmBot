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
    pub const fn new(id: u32) -> Self {
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
