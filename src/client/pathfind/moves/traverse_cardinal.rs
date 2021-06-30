/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::pathfind::moves::{CardinalDirection, Move};

struct MoveCardinal {
    direction: CardinalDirection,
}

impl Move for MoveCardinal {
    fn on_move(&self, from: BlockLocation, context: &Context) -> MoveResult {
        let Change { dx, dz } = self.direction.unit_change();
        let BlockLocation(x, y, z) = from;

        let legs_loc = BlockLocation(x + dx, y, z + dz);
        let to_legs = context.world.get_block(legs_loc).await;
        let to_head = get_block!(context, x + dx, y, z + dz);
        let to_floor = get_block!(context, x + dx, y, z + dz);

        match (to_floor, to_legs, to_head) {
            (Some(floor), Some(legs), Some(head)) => {
                if floor.full_block() && legs.walk_through() && head.walk_through() {
                    MoveResult::Realized(Realized {
                        result_location: legs_loc,
                        cost: 1.0,
                    })
                } else {
                    MoveResult::Invalid
                }
            }
            _ => MoveResult::Edge,
        }
    }
}
