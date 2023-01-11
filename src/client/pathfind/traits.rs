//! Traits used for path finding

use std::fmt::{Debug, Formatter};

use crate::client::pathfind::context::MoveNode;

/// a heuristic we can use for path finding.
///
/// - It must follow A* heuristic requirements
pub trait Heuristic<T = MoveNode> {
    /// Given an input return a heuristic
    fn heuristic(&self, input: &T) -> f64;
}

/// A neighbor node
pub struct Neighbor<T> {
    /// the value that the neighbor is
    pub value: T,

    /// the differential cost to get to that node
    pub cost: f64,
}

impl<T: Debug> Debug for Neighbor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Neighbor {:?} @ dist {}",
            self.value, self.cost
        ))
    }
}

/// The returned path
pub enum Progression<T> {
    /// we are on an edge (TODO: what??) and cannot find a path
    Edge,

    /// a list of movements. This is essentially the found path
    Movements(Vec<Neighbor<T>>),
}

impl<T: Debug> Debug for Progression<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Edge => f.write_str("Edge"),
            Self::Movements(neighbors) => neighbors.fmt(f),
        }
    }
}
/// A [`Progressor`] is a trait which can given an input find a path
///
/// A [`Progressor`] generally has a notion of a goal built into it
pub trait Progressor<T = MoveNode> {
    /// find progressions towards a goal given an `input`
    fn progressions(&self, input: &T) -> Progression<T>;
}

/// The ability to tell whether a node is a goal node or not
pub trait GoalCheck<T = MoveNode> {
    /// determine if `input` is a goal
    fn is_goal(&self, input: &T) -> bool;
}
