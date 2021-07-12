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
