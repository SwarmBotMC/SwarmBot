use std::fmt::{Debug, Formatter};

use crate::client::pathfind::context::Context;
use crate::client::pathfind::moves::{Movements};
use crate::storage::block::BlockLocation;

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
    pub cost: f64,
}

impl<T: Debug> Debug for Neighbor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Neighbor {:?} @ dist {}", self.value, self.cost))
    }
}

pub enum Progression<T> {
    Edge,
    Movements(Vec<Neighbor<T>>),
}

pub trait Progressor {
    type Unit;
    fn progressions(&self, input: Self::Unit) -> Progression<Self::Unit>;
}


#[derive(Clone)]
pub struct ProgressChecker<'a, 'b: 'a> {
    ctx: &'a Context<'b>,
}

impl Progressor for ProgressChecker<'_, '_> {
    type Unit = BlockLocation;

    fn progressions(&self, location: BlockLocation) -> Progression<BlockLocation> {
        Movements::obtain_all(location, self.ctx)
    }
}

impl<'a, 'b: 'a> ProgressChecker<'a, 'b> {
    pub fn new(ctx: &'a Context<'b>) -> ProgressChecker<'a, 'b> {
        ProgressChecker {
            ctx
        }
    }
}
