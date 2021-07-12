/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
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
