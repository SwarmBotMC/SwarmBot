/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */




use crate::client::pathfind::context::{GlobalContext, MoveNode};
use crate::client::pathfind::moves::cenetered_arr::CenteredArray;
use crate::client::pathfind::traits::{Neighbor, Progression};
use crate::storage::block::{BlockLocation, SimpleType};
use crate::storage::blocks::WorldBlocks;

pub const MAX_FALL: i32 = 3;

mod cenetered_arr;

#[derive(Copy, Clone, Eq, PartialEq)]
enum State {
    Open,
    Closed,
}

impl Default for State {
    fn default() -> Self {
        Self::Open
    }
}

pub struct Movements;

impl Movements {

    pub fn obtain_all(on: &MoveNode, ctx: &GlobalContext) -> Progression<MoveNode> {
        let BlockLocation { x, y, z } = on.location;
        let w = ctx.world;

        macro_rules! get_block {
            ($x: expr, $y: expr, $z:expr) => {{
                let res: Option<SimpleType> = w.get_block_simple(BlockLocation::new($x,$y,$z));
                res
            }};
        }


        // macro_rules! get_kind {
        //     ($x: expr, $y: expr, $z:expr) => {{
        //         let res: Option<BlockKind> = w.get_block_kind(BlockLocation::new($x,$y,$z));
        //         res
        //     }};
        // }

        macro_rules! wrap {
            ($block_loc: expr) => {{
                let mut node = MoveNode::from(&on);
                node.location = $block_loc;
                node
            }};
        }

        let (head, multiplier) = match get_block!(x, y + 1, z) {
            None => return Progression::Edge,
            Some(inner) => {
                // we do not like our head in water (breathing is nice)
                let multiplier = if inner == Water { ctx.path_config.costs.no_breathe_mult } else { 1.0 };
                (inner, multiplier)
            }
        };


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

        // what we are going to turn for progressions
        let mut res = vec![];

        let mut traverse_possible_no_place = [false; 4];

        // moving adjacent without changing elevation
        for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();
            if can_move_adj_noplace[idx] {
                let floor = get_block!(x + dx, y - 1, z + dz).unwrap();
                let walkable = floor == Solid || adj_legs[idx] == Water || adj_head[idx] == Water;
                traverse_possible_no_place[idx] = walkable;
                if walkable {
                    res.push(Neighbor {
                        value: wrap!(BlockLocation::new(x + dx, y, z + dz)),
                        cost: ctx.path_config.costs.block_walk * multiplier,
                    })
                }
            }
        }

        // descending adjacent
        for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
            let Change { dx, dz, .. } = direction.unit_change();

            let floor = get_block!(x + dx, y - 1, z + dz).unwrap();
            if can_move_adj_noplace[idx] && !traverse_possible_no_place[idx] && floor != Avoid {
                let start = BlockLocation::new(x + dx, y, z + dz);
                let collided_y = drop_y(start, w);
                if let Some(collided_y) = collided_y {
                    let new_pos = BlockLocation::new(x + dx, collided_y + 1, z + dz);

                    res.push(Neighbor {
                        value: wrap!(new_pos),
                        cost: ctx.path_config.costs.fall * multiplier,
                    })
                }
            }
        }

        let above = get_block!(x, y + 2, z).unwrap();
        let floor = get_block!(x, y - 1, z).unwrap();
        let feet = get_block!(x, y, z).unwrap();

        if above == Water || head == Water && above == WalkThrough {
            res.push(Neighbor {
                value: wrap!(BlockLocation::new(x,y+1,z)),
                cost: ctx.path_config.costs.ascend * multiplier,
            });
        }

        if floor == Water || (floor == WalkThrough && head == Water) {
            res.push(Neighbor {
                value: wrap!(BlockLocation::new(x,y-1,z)),
                cost: ctx.path_config.costs.ascend * multiplier,
            });
        }


        let can_micro_jump = above == WalkThrough && (floor == Solid || feet == Water);

        if can_micro_jump {
            // ascending adjacent
            for (idx, direction) in CardinalDirection::ALL.iter().enumerate() {
                let Change { dx, dz, .. } = direction.unit_change();

                // we can only move if we couldn't move adjacent without changing elevation
                if !can_move_adj_noplace[idx] {
                    let adj_above = matches!(get_block!(x+dx, y+2, z+dz).unwrap(), WalkThrough | Water);
                    let can_jump = adj_above && adj_legs[idx] == Solid && matches!(adj_head[idx], WalkThrough | Water);
                    if can_jump {
                        res.push(Neighbor {
                            value: wrap!(BlockLocation::new(x+dx,y+1,z+dz)),
                            cost: ctx.path_config.costs.ascend * multiplier,
                        });
                    }
                }
            }
        }

        // can full multi-block jump (i.e., jumping on bedrock)
        let can_jump = above == WalkThrough && floor != Water;

        if can_jump {
            // we can jump in a 3 block radius

            const RADIUS: i32 = 4;
            const RADIUS_S: usize = RADIUS as usize;

            // let mut not_jumpable = SmallVec::<[_; RADIUS_S * RADIUS_S]>::new();
            let mut not_jumpable = Vec::new();
            let mut edge = false;


            'check_loop:
            for dx in -RADIUS..=RADIUS {
                for dz in -RADIUS..=RADIUS {
                    let adj_above = get_block!(x+dx, y+2, z+dz);
                    if adj_above == None {
                        edge = true;
                        break 'check_loop;
                    }

                    let adj_above = adj_above.unwrap() == WalkThrough;
                    let adj_head = get_block!(x+dx, y+1, z+dz).unwrap() == WalkThrough;
                    let adj_feet = get_block!(x+dx, y, z+dz).unwrap() == WalkThrough;
                    if !(adj_above && adj_head && adj_feet) {
                        not_jumpable.push((dx, dz));
                    }
                }
            }

            if edge {
                return Progression::Edge;
            }


            let mut open = CenteredArray::init::<_, RADIUS_S>();

            // so we do not add the origin (it is already added)
            open[(0, 0)] = State::Closed;

            // we iterate through every single block which is not jumpable and set blocks behind it as not jumpable as well
            for (block_dx, block_dz) in not_jumpable {




                // we will set blocks to closed in the direction of the block

                let mut update = |sign_x: i32, sign_z: i32| {
                    let increments = RADIUS - block_dx.abs().max(block_dz.abs()) + 1;

                    for inc in 0..increments {
                        let dx = block_dx + inc * sign_x;
                        let dz = block_dz + inc * sign_z;
                        open[(dx, dz)] = State::Closed;
                        if dx.abs() < RADIUS {
                            open[(dx + sign_x, dz)] = State::Closed;
                        }

                        if dz.abs() < RADIUS {
                            open[(dx, dz + sign_z)] = State::Closed;
                        }
                    }
                };

                let sign_x = block_dx.signum();
                let sign_z = block_dz.signum();

                if block_dx == 0 {
                    // special case: we need to update blocks in both directions
                    update(-1, sign_z);
                    update(0, sign_z);
                    update(1, sign_z);
                } else if block_dz == 0 {
                    // special case: we need to update blocks in both directions
                    update(sign_x, -1);
                    update(sign_x, 0);
                    update(sign_x, 1);
                } else {
                    // we only update blocks in the direction it is in
                    update(sign_x, sign_z);
                }
            }

            for dx in -RADIUS..=RADIUS {
                for dz in -RADIUS..=RADIUS {
                    let is_open = open[(dx, dz)] == State::Open;

                    let same_y = get_block!(x+dx, y - 1, z+dz).unwrap();

                    let same_y_possible = same_y == Solid;

                    let rad2 = (dx * dx + dz * dz) as f64;

                    const MIN_RAD: f64 = 2.0;
                    const MAX_RAD: f64 = 4.5;

                    if same_y_possible && rad2 <= MAX_RAD * MAX_RAD && rad2 > MIN_RAD * MIN_RAD && is_open {
                        res.push(Neighbor {
                            value: wrap!(BlockLocation::new(x+dx,y,z+dz)),
                            cost: ctx.path_config.costs.block_parkour * multiplier,
                        });
                    }
                }
            }
        }


        Progression::Movements(res)
    }
}

fn drop_y(start: BlockLocation, world: &WorldBlocks) -> Option<i16> {
    let BlockLocation { x, y: init_y, z } = start;

    // only falling we could do would be into the void
    if init_y < 2 {
        return None;
    }

    let mut travelled = 1;
    for y in (0..=(init_y - 2)).rev() {
        let loc = BlockLocation::new(x, y, z);
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


    None
}

#[derive(Copy, Clone, Debug)]
pub enum CardinalDirection {
    North,
    South,
    West,
    East,
}

impl CardinalDirection {
    pub const ALL: [CardinalDirection; 4] = {
        use CardinalDirection::*;
        [North, South, East, West]
    };
}

pub struct Change {
    pub dx: i32,
    pub dy: i16,
    pub dz: i32,
}

impl Change {
    fn new(dx: i32, dy: i16, dz: i32) -> Change {
        Change { dx, dy, dz }
    }
}


impl CardinalDirection {
    pub fn unit_change(&self) -> Change {
        match self {
            CardinalDirection::North => Change::new(1, 0, 0),
            CardinalDirection::South => Change::new(-1, 0, 0),
            CardinalDirection::West => Change::new(0, 0, 1),
            CardinalDirection::East => Change::new(0, 0, -1)
        }
    }
}
