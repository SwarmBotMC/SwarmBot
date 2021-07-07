/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/7/21, 12:20 AM
 */

use crate::protocol::InterfaceOut;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::tasks::Task;

pub trait TaskStream {
    fn poll(&mut self, out: &mut impl InterfaceOut, local: &mut LocalState, global: &mut GlobalState) -> Option<Task>;
}
