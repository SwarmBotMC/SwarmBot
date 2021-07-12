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

use crate::protocol::InterfaceOut;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::Task;

pub trait TaskStream {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task>;
}
