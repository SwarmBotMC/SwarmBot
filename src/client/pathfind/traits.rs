/*
 * Copyright (c) 2021 Minecraft IGN RevolutionNow - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by RevolutionNow <Xy8I7.Kn1RzH0@gmail.com>, 6/29/21, 8:16 PM
 */

use std::fmt::{Debug, Formatter};

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

impl<T: Debug> Debug for Progression<T> {
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


pub trait GoalCheck<T> {
    fn is_goal(&self, input: &T) -> bool;
}
