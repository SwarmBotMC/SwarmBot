use async_trait::async_trait;

use crate::data::{BlockState, AIR};
use crate::pathfind::context::Context;
use crate::pathfind::moves::Movements::TraverseCardinal;
use crate::pathfind::BlockLocation;
use std::sync::Arc;
use super::BlockLocation;
use crate::client::pathfind::context::Context;
use crate::client::pathfind::moves::Movements::TraverseCardinal;

trait Move {
    #[async_trait]
    fn on_move(&self, from: BlockLocation, context: &Context) -> MoveResult;
}

struct Realized {
    result_location: BlockLocation,
    cost: f64,
}

enum MoveResult {
    Edge,
    Invalid,
    Realized(Realized),
}

enum MoveResults {
    Edge,
    Realized(Vec<Realized>),
}

pub enum Movements {
    TraverseCardinal(CardinalDirection),
}

impl Movements {
    const ALL: Vec<Movements> = {
        use crate::pathfind::moves::Movements::*;
        vec![
            TraverseCardinal(CardinalDirection::NORTH),
            TraverseCardinal(CardinalDirection::WEST),
            TraverseCardinal(CardinalDirection::SOUTH),
            TraverseCardinal(CardinalDirection::EAST),
        ]
    };

    async fn obtain_all(from: BlockLocation, ctx: &Context) -> MoveResults {
        // type OptBS = Optional<BlockState>;

        macro_rules! get_block {
            ($x: expr, $y:expr, $z:expr) => {{
                let res: Option<BlockState> = ctx.world.get_block(BlockLocation($x, $y, $z)).await;
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
            let head = get_block!(x + dx, y + 1, z + dz);

            let res = match (legs, head) {
                (Some(legs), Some(head)) => {
                    adj_legs[idx] = legs;
                    adj_head[idx] = head;
                    can_move_adj[idx] = legs.walk_through() || head.walk_through();
                }
                _ => return MoveResults::Edge,
            };

        }

        let mut res = vec![];

        let mut can_traverse_no_block = [false; 4];

        let current_floor = get_block!(x, y, z).unwrap();

        let mut adj_floor = [AIR; 4];

        // traversing
        for (idx, direction) in CardinalDirection::ALL.into_iter().enumerate() {
            let CardinalDirection { dx, dz } = direction;
            if can_move_adj[idx] {
                let floor = get_block!(ctx, x + dx, y - 1, z + dz).unwrap();
                let floor_walkable = floor.full_block();
                can_traverse_no_block[idx] = floor_walkable;
                res.push(Realized {
                    result_location: BlockLocation(x + dx, y - 1, z + dz),
                    cost: ctx.costs.block_walk,
                })
            }
        }

        // falling
        for (idx, direction) in CardinalDirection::ALL.into_iter().enumerate() {
            let CardinalDirection { dx, dz } = direction;
            if can_move_adj[idx] {
                let floor = get_block!(ctx, x + dx, y - 1, z + dz).unwrap();
                can_traverse_no_block[idx] = floor.full_block();
            }
        }

        todo!()
    }
}

struct MovementCache {}

impl Move for Movements {
    fn on_move(&self, from: BlockLocation, context: &Context) -> MoveResult {
        match self {
            TraverseCardinal(&direction) => MoveCardinal { direction }.on_move(from, context),
            _ => {
                panic!("dasd")
            }
        }
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
    pub const ALL: Vec<CardinalDirection3D> = {
        use CardinalDirection::*;
        use CardinalDirection3D::*;
        vec![
            Plane(NORTH),
            Plane(SOUTH),
            Plane(EAST),
            Plane(WEST),
            DOWN,
            UP,
        ]
    };

    pub const ALL_BUT_UP: Vec<CardinalDirection3D> = {
        use CardinalDirection::*;
        use CardinalDirection3D::*;
        vec![Plane(NORTH), Plane(SOUTH), Plane(EAST), Plane(WEST), DOWN]
    };
}

impl CardinalDirection {
    pub const ALL: Vec<CardinalDirection> = {
        use CardinalDirection::*;
        vec![NORTH, SOUTH, EAST, WEST]
    };
}

pub struct Change {
    pub dx: u8,
    pub dy: u8,
    pub dz: u8,
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
