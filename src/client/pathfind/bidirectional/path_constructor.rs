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

use std::collections::HashMap;
use std::hash::Hash;

pub struct PathConstructor;


fn path_trace<T: Copy + Hash + Eq>(from: T, lookup: &HashMap<T, T>, into: &mut Vec<T>) {
    let mut on = from;
    into.push(on);
    while let Some(&prev) = lookup.get(&on) {
        into.push(prev);
        on = prev;
    }
}

impl PathConstructor {
    pub fn build_path<T: Copy + Hash + Eq>(forward: &HashMap<T, T>, backward: &HashMap<T, T>, split: T) -> Vec<T> {
        let mut vec = Vec::new();
        path_trace(split, forward, &mut vec);
        vec.reverse();
        path_trace(split, backward, &mut vec);
        vec
    }

    pub fn build_path_forward<T: Copy + Hash + Eq>(forward: &HashMap<T, T>, goal: T) -> Vec<T> {
        let mut vec = Vec::new();
        path_trace(goal, forward, &mut vec);
        vec.reverse();
        vec
    }
}
