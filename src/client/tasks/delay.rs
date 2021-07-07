/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:08 AM
 */

use crate::client::tasks::TaskTrait;
use crate::protocol::InterfaceOut;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;

pub struct DelayTask(pub u32);

impl DelayTask {
    pub fn new(ticks: u32) -> Self {
        Self(ticks)
    }
}

impl TaskTrait for DelayTask {
    fn tick(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> bool {
        let ticks_left = self.0;
        if ticks_left == 0 {
            true
        } else {
            self.0 -= 1;
            false
        }
    }
}
