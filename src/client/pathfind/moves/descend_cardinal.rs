/*
 * Copyright (c) 2021 Minecraft IGN RevolutionNow - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by RevolutionNow <Xy8I7.Kn1RzH0@gmail.com>, 6/29/21, 8:16 PM
 */

use crate::pathfind::BlockLocation;
use crate::pathfind::context::Context;
use crate::pathfind::moves::{Move, MoveResult};

struct DescendCardinal;

impl Move for DescendCardinal {
    fn on_move(&self, from: BlockLocation, context: &Context) -> MoveResult {
        todo!()
    }
}
