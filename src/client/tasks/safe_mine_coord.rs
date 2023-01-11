use interfaces::types::{BlockLocation, BlockLocation2D};

use crate::client::{
    state::{
        global::{mine_alloc::MineAlloc, GlobalState},
        local::LocalState,
    },
    tasks::{
        center::CenterTask,
        compound::CompoundTask,
        delay::DelayTask,
        lazy::{Lazy, LazyTask},
        lazy_stream::LazyStream,
        mine_column::MineColumn,
        mine_goto::GoMineTop,
        Task,
    },
};

pub struct SafeMineRegion;

impl Lazy for SafeMineRegion {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task {
        let location = BlockLocation::from(local.physics.location());
        let center = BlockLocation2D::from(location);

        // if we should skip this region. For example, if there is water or lava we will
        // want to avoid it
        let avoid = MineAlloc::locations_extra(center).any(|loc| {
            // there is often lava under bedrock that we don't really care about
            if loc.y < MineColumn::MIN_MINE_LOC {
                return false;
            }

            match global.blocks.get_block_exact(loc).map(|x| x.kind().id()) {
                // water or lava
                Some(8..=11) => {
                    println!(
                        "skipping region {}, {} because of {:?} at {}",
                        center.x,
                        center.z,
                        global.blocks.get_block_exact(loc),
                        loc
                    );
                    true
                }
                _ => false,
            }
        });

        if avoid {
            DelayTask(0).into()
        } else {
            let mut compound = CompoundTask::default();

            compound
                .add(CenterTask)
                .add(LazyTask::from(GoMineTop))
                .add(LazyStream::from(MineColumn));

            compound.into()
        }
    }
}
