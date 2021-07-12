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

use std::fmt::{Debug, Formatter};
use crate::client::pathfind::context::MoveNode;

pub trait Heuristic<T = MoveNode> {
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

impl<T: Debug> Debug for Progression<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Progression::Edge => f.write_str("Edge"),
            Progression::Movements(neighbors) => neighbors.fmt(f)
        }
    }
}

pub trait Progressor<T=MoveNode> {
    fn progressions(&self, input: &T) -> Progression<T>;
}


pub trait GoalCheck<T=MoveNode> {
    fn is_goal(&self, input: &T) -> bool;
}
