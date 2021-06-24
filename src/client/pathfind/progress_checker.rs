use std::fmt::{Debug, Formatter};

use crate::client::pathfind::context::{MoveNode, GlobalContext};
use crate::client::pathfind::moves::{Movements};
use crate::storage::block::BlockLocation;

pub trait Heuristic<T> {
    fn heuristic(&self, input: &T) -> f64;
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

impl <T: Debug> Debug for Progression<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Progression::Edge => f.write_str("Edge"),
            Progression::Movements(neighbors) => neighbors.fmt(f)
        }
    }
}


pub trait Progressor<T> {
    fn progressions(&self, input: &T) -> Progression<T>;
}

pub struct NoVehicleHeuristic {
    pub move_cost: f64,
    pub goal: BlockLocation
}

impl Heuristic<MoveNode> for NoVehicleHeuristic {
    fn heuristic(&self, input: &MoveNode) -> f64 {
        let current = input.location;
        current.dist(self.goal) * self.move_cost
    }
}

pub trait GoalCheck<T> {
    fn is_goal(&self, input: &T) -> bool;
}

pub struct NoVehicleGoalCheck {
    goal: BlockLocation
}

impl NoVehicleGoalCheck {
    pub fn new(goal: BlockLocation) -> Self {
        Self {
            goal
        }
    }
}

impl GoalCheck<MoveNode> for NoVehicleGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        input.location == self.goal
    }
}

#[derive(Clone)]
pub struct NoVehicleProgressor<'a> {
    ctx: GlobalContext<'a>,
}

impl Progressor<MoveNode> for NoVehicleProgressor<'_> {
    fn progressions(&self, location: &MoveNode) -> Progression<MoveNode> {
        Movements::obtain_all(location, &self.ctx)
    }
}

impl<'a> NoVehicleProgressor<'a> {
    pub fn new(ctx: GlobalContext<'a>) -> NoVehicleProgressor<'a> {
        NoVehicleProgressor {
            ctx
        }
    }
}
