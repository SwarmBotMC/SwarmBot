use crate::client::pathfind::context::{GlobalContext, MoveContext};
use crate::client::pathfind::moves::Movements::TraverseCardinal;
use crate::client::pathfind::progress_checker::{Neighbor, Progression};
use crate::storage::block::{BlockLocation, SimpleType};
use crate::storage::blocks::WorldBlocks;

pub const MAX_FALL: i32 = 22;

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

    pub fn obtain_all(on: &MoveContext, ctx: &GlobalContext) -> Progression<MoveContext> {
        let BlockLocation(x, y, z) = on.location;
        let w = ctx.world;
        let blocks_can_place = on.blocks_can_place;

        macro_rules! get_block {
            ($x: expr, $y: expr, $z:expr) => {{
                let res: Option<SimpleType> = w.get_block_simple(BlockLocation($x,$y,$z));
                res
            }};
        }

        macro_rules! wrap {
            ($block_loc: expr) => {{
                MoveContext {
                    blocks_can_place,
                    location: $block_loc
                }
            }};
        }


        use crate::storage::block::SimpleType::*;

        // cache adjacent leg block types
        let mut adj_legs = [WalkThrough; 4];
        let mut adj_head = [WalkThrough; 4];

        // if adj_legs && adj_head is true for any idx
        let mut can_move_adj_noplace = [false; 4];

        for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();

            let legs = get_block!(x + dx, y, z + dz);
            let head = get_block!(x + dx, y + 1, z + dz);

            match (legs, head) {
                (Some(legs), Some(head)) => {
                    adj_legs[idx] = legs;
                    adj_head[idx] = head;
                    can_move_adj_noplace[idx] = matches!(legs, WalkThrough | Water) && matches!(head, WalkThrough | Water);
                }
                _ => return Progression::Edge,
            };
        }

        // what we are going to turn for progressoins
        let mut res = vec![];

        let mut traverse_possible_no_place = [false; 4];

        // moving adjacent without changing elevation
        for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();
            if can_move_adj_noplace[idx] {
                let floor = get_block!(x + dx, y - 1, z + dz).unwrap();
                let floor_walkable = floor == Solid;
                traverse_possible_no_place[idx] = floor_walkable;
                if floor_walkable {
                    res.push(Neighbor {
                        value: wrap!(BlockLocation(x + dx, y, z + dz)),
                        cost: ctx.path_config.costs.block_walk,
                    })
                }
            }
        }

        // descending adjacent
        for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();

            if can_move_adj_noplace[idx] && !traverse_possible_no_place[idx] {
                let start = BlockLocation(x + dx, y, z + dz);
                let collided_y = can_fall(start, w);
                if let Some(collided_y) = collided_y {
                    let new_pos = BlockLocation(x + dx, collided_y + 1, z + dz);

                    res.push(Neighbor {
                        value: wrap!(new_pos),
                        cost: ctx.path_config.costs.fall,
                    })
                }
            }
        }

        let can_jump = matches!(get_block!(x, y + 2, z).unwrap(), WalkThrough | Water);

        // ascending adjacent
        if can_jump {
            for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
                let Change { dx, dz, .. } = direction.unit_change();

                // we can only move if we couldn't move adjacent without changing elevation
                if !can_move_adj_noplace[idx] {
                    let adj_above = get_block!(x+dx, y+2, z+dz).unwrap() == WalkThrough;
                    let can_jump = adj_above && adj_legs[idx] == Solid && adj_head[idx] == WalkThrough;
                    if can_jump {
                        res.push(Neighbor {
                            value: wrap!(BlockLocation(x+dx,y+1,z+dz)),
                            cost: ctx.path_config.costs.ascend,
                        });
                    }
                }
            }
        }

        Progression::Movements(res)
    }
}

fn can_fall(start: BlockLocation, world: &WorldBlocks) -> Option<i64> {
    let BlockLocation(x, init_y, z) = start;

    // only falling we could do would be into the void
    if init_y < 2 {
        return None;
    }

    let mut travelled = 1;
    for y in (0..=(init_y - 2)).rev() {
        let loc = BlockLocation(x, y, z);
        let block_type = world.get_block_simple(loc).unwrap();
        match block_type {
            SimpleType::Solid => {
                return (travelled <= MAX_FALL).then(|| y);
            }
            SimpleType::Water => {
                return Some(y);
            }
            SimpleType::Avoid => {
                return None;
            }
            SimpleType::WalkThrough => {}
        }

        travelled += 1;
    }


    return None;
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

impl Change {
    fn new(dx: i64, dy: i64, dz: i64) -> Change {
        Change { dx, dy, dz }
    }
}

impl CardinalDirection3D {
    pub fn unit_change(&self) -> Change {
        todo!()
    }
}

impl CardinalDirection {
    fn unit_change(&self) -> Change {
        match self {
            CardinalDirection::NORTH => Change::new(1, 0, 0),
            CardinalDirection::SOUTH => Change::new(-1, 0, 0),
            CardinalDirection::WEST => Change::new(0, 0, 1),
            CardinalDirection::EAST => Change::new(0, 0, -1)
        }
    }
}
