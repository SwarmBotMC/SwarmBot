/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:09 AM
 */

use crate::client::tasks::TaskTrait;
use crate::protocol::InterfaceOut;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;

pub struct EatTask {
    ticks: usize,
}

const TICKS: usize = 40;

impl Default for EatTask {
    fn default() -> Self {
        // shouldn't need to be 40 (32... but because of lag I guess it sometimes does)
        Self {ticks: TICKS}
    }
}

impl TaskTrait for EatTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, _: &mut LocalState, _: &mut GlobalState) -> bool {

        // start eating
        if self.ticks == TICKS {
            out.use_item();
        }

        if self.ticks == 0 {
            out.finish_eating();
            true
        } else {
            self.ticks -= 1;
            false
        }
    }
}
