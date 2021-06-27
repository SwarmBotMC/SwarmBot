/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/27/21, 3:15 PM
 */

use crate::pathfind::moves::{Move, MoveResult};
use crate::pathfind::context::Context;
use crate::pathfind::BlockLocation;

struct DescendCardinal;

impl Move for DescendCardinal {
    fn on_move(&self, from: BlockLocation, context: &Context) -> MoveResult {
        todo!()
    }
}
