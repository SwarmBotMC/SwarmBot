/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 4:12 PM
 */


use crate::client::state::global::GlobalState;
use crate::client::state::global::mine_alloc::MineAlloc;
use crate::client::state::local::LocalState;
use crate::client::tasks::lazy::{Lazy, LazyTask};
use crate::client::tasks::Task;
use crate::storage::block::{BlockLocation, BlockLocation2D, SimpleType};
use crate::client::tasks::delay::DelayTask;
use crate::client::tasks::compound::CompoundTask;
use crate::client::tasks::center::CenterTask;
use crate::client::tasks::mine_goto::GoMineTop;
use crate::client::tasks::mine_column::MineColumn;
use crate::client::tasks::lazy_stream::LazyStream;

pub struct SafeMineRegion;

impl Lazy for SafeMineRegion {
    fn create(&self, local: &mut LocalState, global: &GlobalState) -> Task {
        let location = BlockLocation::from(local.physics.location());
        let center = BlockLocation2D::from(location);

        let avoid = MineAlloc::locations(center)
            .any(|loc| global.blocks.get_block_simple(loc) == Some(SimpleType::Avoid));

        if avoid {
            DelayTask(0).into()
        } else {
            let mut compound = CompoundTask::default();

            compound.add(CenterTask)
                .add(LazyTask::from(GoMineTop))
                .add(LazyStream::from(MineColumn::default()));

            compound.into()

        }


    }
}
