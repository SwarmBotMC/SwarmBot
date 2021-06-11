use std::fmt::{Debug, Formatter};

use async_trait::async_trait;

use crate::data::{AIR, BlockState};
use crate::pathfind::BlockLocation;
use crate::pathfind::world_blocks::WorldBlocks;

pub trait Heuristic<T> {
    fn heuristic(&self, input: &T, goal_node: &T) -> f64;
}

pub struct BlockHeuristic;

impl Heuristic<BlockLocation> for BlockHeuristic {
    fn heuristic(&self, input: &BlockLocation, goal_node: &BlockLocation) -> f64 {
        input.dist(*goal_node)
    }
}

pub struct Neighbor<T> {
    pub value: T,
    pub distance: f64,
}

impl<T: Debug> Debug for Neighbor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Neighbor {:?} @ dist {}", self.value, self.distance))
    }
}

pub enum Progression<T> {
    Edge,
    Movements(Vec<Neighbor<T>>),
}

#[async_trait]
pub trait Progressor {
    type Unit;
    async fn progressions(&self, input: Self::Unit) -> Progression<Self::Unit>;
}


#[derive(Clone)]
pub struct ProgressChecker<'a> {
    storage: &'a WorldBlocks,
}

#[async_trait]
impl Progressor for ProgressChecker<'_> {
    type Unit = BlockLocation;

    async fn progressions(&self, location: BlockLocation) -> Progression<BlockLocation> {
        let BlockLocation(x, y, z) = location;

        let locations = vec![
            // adjacent
            BlockLocation(x + 1, y, z),
            BlockLocation(x - 1, y, z),
            BlockLocation(x, y, z + 1),
            BlockLocation(x, y, z - 1),

            // adjacent below
            BlockLocation(x + 1, y - 1, z),
            BlockLocation(x - 1, y - 1, z),
            BlockLocation(x, y - 1, z + 1),
            BlockLocation(x, y - 1, z - 1),

            // adjacent above
            BlockLocation(x + 1, y + 1, z),
            BlockLocation(x - 1, y + 1, z),
            BlockLocation(x, y + 1, z + 1),
            BlockLocation(x, y + 1, z - 1),

            // adjacent far below
            BlockLocation(x + 1, y - 2, z),
            BlockLocation(x - 1, y - 2, z),
            BlockLocation(x, y - 2, z + 1),
            BlockLocation(x, y - 2, z - 1),

            // above
            BlockLocation(x, y + 1, z),
        ];

        let blocks = self.storage.get_blocks(locations).await;

        let on_edge = blocks.iter().take(4).any(|block| block.is_none());

        if on_edge {
            return Progression::Edge;
        }

        let possibilities = Vec::new();

        macro_rules! try_add {
            }
    };
}

for i in 0..4 {

// moving adjacently
let legs = blocks[i].unwrap();
let floor = blocks[i + 4].unwrap();
let above = blocks[i + 8].unwrap();
let below_floor = blocks[i + 12].unwrap();

// move adjacent
try_add ! (floor, legs, above);

if (floor.walk_through() & & )
}


let res = possibilities.into_iter().map( | to_loc| Neighbor {
value: to_loc,
distance: to_loc.dist(location),
}).collect();

Progression::Movements(res)
}
}

impl ProgressChecker<'_> {
    pub fn new(storage: &'_ WorldBlocks) -> ProgressChecker<'_> {
        ProgressChecker {
            storage
        }
    }

    #[inline]
    fn try_add(&self, floor: BlockState, legs: BlockState, head: BlockState, below_floor: BlockState, above_head: BlockState, possibilities: &mut Vec<BlockLocation>) {

        // we cannot walk
        if legs.full_block() || head.full_block() { return; }

        // we can go to floor
        if floor.full_block() && legs.walk_through() && head.walk_through() {

        }
        // if floor.full_block() && legs.walk_through() && head.walk_through() {
        //     possibilities.push(legs);
        // } else if (floor.is_water() || (floor.is) && head.walk_through() {
        //     possibilities.push(legs);
        // } else if head.is_water() && floor.walk_through() {
        //     possibilities.push(legs);
        // }
    }
}
