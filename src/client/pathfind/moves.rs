use crate::client::pathfind::context::Context;
use crate::client::pathfind::moves::Movements::TraverseCardinal;
use crate::client::pathfind::progress_checker::{Neighbor, Progression};
use crate::storage::block::{AIR, BlockApprox, BlockLocation, SimpleType};
use crate::storage::block::SimpleType::WalkThrough;

enum MoveResult {
    Edge,
    Invalid,
    Realized(Neighbor<BlockLocation>),
}

pub enum Movements {
    TraverseCardinal(CardinalDirection),
}

impl Movements {
    const ALL: [Movements; 4] = {
        [
            TraverseCardinal(CardinalDirection::NORTH),
            TraverseCardinal(CardinalDirection::WEST),
            TraverseCardinal(CardinalDirection::SOUTH),
            TraverseCardinal(CardinalDirection::EAST),
        ]
    };

    pub fn obtain_all(from: BlockLocation, ctx: &Context) -> Progression<BlockLocation> {
        let w = ctx.world;

        macro_rules! get_block {
            ($x: expr, $y: expr, $z:expr) => {{
                let res: Option<BlockApprox> = w.get_block(BlockLocation($x,$y,$z));
                res
            }};
        }

        let mut can_move_adj = [false; 4];

        let mut adj_legs = [AIR; 4];
        let mut adj_head = [AIR; 4];

        let BlockLocation(x, y, z) = from;

        // let edge_blocks = [B]
        // movement directions
        for (idx, direction) in CardinalDirection::ALL.into_iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();

            let legs = get_block!(x + dx, y, z + dz);
            let head = get_block!(x + dx, y, z + dz);

            match (legs, head) {
                (Some(legs), Some(head)) => {
                    adj_legs[idx] = legs;
                    adj_head[idx] = head;
                    can_move_adj[idx] = legs.s_type() == WalkThrough && head.s_type() == WalkThrough;
                }
                _ => return Progression::Edge,
            };
        }

        let mut res = vec![];

        let mut can_traverse_no_block = [false; 4];

        let current_floor = get_block!(x, y, z).unwrap();

        let mut adj_floor = [AIR; 4];

        // traversing
        for (idx, direction) in CardinalDirection::ALL.into_iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();
            if can_move_adj[idx] {
                let floor = get_block!(x + dx, y - 1, z + dz).unwrap();
                let floor_walkable = floor.s_type() == SimpleType::Solid;
                can_traverse_no_block[idx] = floor_walkable;
                res.push(Neighbor {
                    value: BlockLocation(x + dx, y - 1, z + dz),
                    cost: ctx.costs.block_walk,
                })
            }
        }

        // falling
        for (idx, direction) in CardinalDirection::ALL.into_iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();
            if can_move_adj[idx] {
                let floor = get_block!(x + dx, y - 1, z + dz).unwrap();
                can_traverse_no_block[idx] = floor.s_type() == SimpleType::Solid;
            }
        }

        Progression::Movements(res)
    }
}

pub enum CardinalDirection {
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

pub enum CardinalDirection3D {
    Plane(CardinalDirection),
    UP,
    DOWN,
}

impl CardinalDirection3D {
    pub const ALL: [CardinalDirection3D; 6] = {
        use CardinalDirection::*;
        use CardinalDirection3D::*;
        [
            Plane(NORTH),
            Plane(SOUTH),
            Plane(EAST),
            Plane(WEST),
            DOWN,
            UP,
        ]
    };

    pub const ALL_BUT_UP: [CardinalDirection3D; 5] = {
        use CardinalDirection::*;
        use CardinalDirection3D::*;
        [Plane(NORTH), Plane(SOUTH), Plane(EAST), Plane(WEST), DOWN]
    };
}

impl CardinalDirection {
    pub const ALL: [CardinalDirection; 4] = {
        use CardinalDirection::*;
        [NORTH, SOUTH, EAST, WEST]
    };
}

pub struct Change {
    pub dx: i64,
    pub dy: i64,
    pub dz: i64,
}

impl CardinalDirection3D {
    pub fn unit_change(&self) -> Change {
        todo!()
    }
}

impl CardinalDirection {
    fn unit_change(&self) -> Change {
        todo!()
    }
}
