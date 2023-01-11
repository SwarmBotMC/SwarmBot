use std::collections::HashSet;

use crate::{
    client::{
        physics::Physics,
        state::{global::GlobalState, local::LocalState},
        tasks::{
            lazy_stream::LazyStream, mine::MineTask, pillar::PillarTask, stream::TaskStream, Task,
        },
    },
    protocol::{Face, InterfaceOut},
    types::Displacement,
};

#[allow(clippy::module_name_repetitions)]
pub type PillarAndMineTask = LazyStream<PillarOrMine>;

impl PillarAndMineTask {
    pub fn pillar_and_mine(height: u32) -> Self {
        let state = PillarOrMine { height };
        Self::from(state)
    }
}

pub struct PillarOrMine {
    height: u32,
}

impl TaskStream for PillarOrMine {
    fn poll(
        &mut self,
        out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> Option<Task> {
        let current_height =
            u32::try_from((local.physics.location().y).floor() as i64).unwrap_or_default();

        // > not >= because we are considering block height
        if current_height > self.height {
            return None;
        }

        let above1 = local.physics.location() + Displacement::new(0., 2.5, 0.);
        let mut set = HashSet::new();
        Physics::in_cross_section(above1, &global.blocks, &mut set);

        macro_rules! mine_task {
            ($position:expr) => {{
                let mut task = MineTask::new($position, out, local, global);
                task.set_face(Face::NegY);
                Some(task.into())
            }};
        }

        if let Some(&position) = set.iter().next() {
            mine_task!(position)
        } else {
            let above2 = local.physics.location() + Displacement::new(0., 3.5, 0.);
            Physics::in_cross_section(above2, &global.blocks, &mut set);
            if let Some(&position) = set.iter().next() {
                mine_task!(position)
            } else {
                local.inventory.switch_block(out);
                Some(PillarTask::new(current_height + 1).into())
            }
        }
    }
}
