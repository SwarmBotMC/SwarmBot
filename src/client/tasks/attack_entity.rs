use interfaces::types::{BlockLocation, BlockLocation2D};

use crate::{
    client::{
        pathfind::implementations::no_vehicle::TravelProblem,
        state::{global::GlobalState, local::LocalState},
        tasks::{
            compound::CompoundTask, delay::DelayTask, hit_entity::HitEntityTask,
            navigate::NavigateProblem, stream::TaskStream, Task,
        },
    },
    protocol::InterfaceOut,
};

pub struct AttackEntity {
    id: u32,
}

impl AttackEntity {
    pub const fn new(id: u32) -> Self {
        Self { id }
    }
}

impl TaskStream for AttackEntity {
    fn poll(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> Option<Task> {
        let current_location = local.physics.location();

        // we cannot do anything if we do not know the location so we end the task
        let entity_location = global.entities.by_id(self.id)?.location;

        let dist2 = entity_location.dist2(current_location);

        const THRESHOLD_DIST: f64 = 3.0;
        const THRESHOLD_DIST_SMALLER: f64 = THRESHOLD_DIST - 0.5;

        if dist2 < THRESHOLD_DIST * THRESHOLD_DIST {
            // we can hit the entity
            let hit = HitEntityTask::new(self.id);
            let mut compound = CompoundTask::default();

            compound.add(hit).add(DelayTask(10));

            Some(compound.into())
        } else {
            // we need to travel to them
            let travel = TravelProblem::navigate_near_block(
                current_location.into(),
                BlockLocation2D::from(BlockLocation::from(entity_location)),
                THRESHOLD_DIST_SMALLER * THRESHOLD_DIST_SMALLER,
                false,
            );
            let task = NavigateProblem::from(travel);

            Some(task.into())
        }
    }
}
