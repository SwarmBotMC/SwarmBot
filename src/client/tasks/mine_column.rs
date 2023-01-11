use interfaces::types::BlockLocation;

use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::{
            compound::CompoundTask, delay::DelayTask, fall_bucket::FallBucketTask, lazy::LazyTask,
            lazy_stream::LazyStream, mine_goto::GoMineTop, mine_layer::MineLayer,
            stream::TaskStream, Task,
        },
    },
    protocol::InterfaceOut,
};

pub struct MineColumn;

impl MineColumn {
    pub const MIN_MINE_LOC: i16 = 11;
}

impl TaskStream for MineColumn {
    fn poll(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        _global: &mut GlobalState,
    ) -> Option<Task> {
        let mine_loc = BlockLocation::from(local.physics.location()).below();
        if mine_loc.y >= Self::MIN_MINE_LOC {
            let mut compound = CompoundTask::default();

            compound
                .add(LazyStream::from(MineLayer))
                .add(DelayTask(5))
                .add(FallBucketTask::default())
                .add(LazyTask::from(GoMineTop));

            Some(compound.into())
        } else {
            None
        }
    }
}

pub type MineColumnTask = LazyStream<MineColumn>;
