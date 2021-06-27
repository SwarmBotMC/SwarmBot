/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/27/21, 3:15 PM
 */

pub enum Increment<T> {
    Finished(T),
    InProgress,
}


/// Tasks which can be done incrementally. Used in pathfinding
pub trait Incremental<T> {
    /// complete an iteration. Returns Some(T) when it is finished
    fn iterate(&mut self) -> Increment<T>;
}
