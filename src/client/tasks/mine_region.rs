use crate::{
    client::{
        pathfind::implementations::no_vehicle::TravelProblem,
        state::{global::GlobalState, local::LocalState},
        tasks::{
            compound::CompoundTask, lazy::LazyTask, navigate::NavigateProblem,
            safe_mine_coord::SafeMineRegion, stream::TaskStream, Task,
        },
    },
    protocol::InterfaceOut,
};

pub struct MineRegion;

impl TaskStream for MineRegion {
    fn poll(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> Option<Task> {
        let goal = global.mine.obtain_region()?;
        let start = local.physics.location();

        let mut compound = CompoundTask::default();
        let problem = TravelProblem::navigate_near_block(start.into(), goal, 0.0, false);
        let nav = NavigateProblem::from(problem);

        compound.add(nav).add(LazyTask::from(SafeMineRegion));

        Some(compound.into())
    }
}
