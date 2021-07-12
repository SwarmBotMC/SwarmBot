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

use std::ops::{Index, IndexMut};

pub struct ConstCenteredArray<T, const SIZE: usize> {
    arr: [T; SIZE],
    width: usize,
}

pub struct CenteredArray;

impl CenteredArray {
    pub fn init<T: Default + Copy, const R: usize>() -> ConstCenteredArray<T, { (2 * R + 1) * (2 * R + 1) }> {
        let arr = [T::default(); (2 * R + 1) * (2 * R + 1)];
        let width: usize = 2 * R + 1;
        ConstCenteredArray {
            arr,
            width,
        }
    }
}

impl<T, const SIZE: usize> ConstCenteredArray<T, SIZE> {
    fn get_idx(&self, x: i32, y: i32) -> usize {
        let size = SIZE as i32;
        debug_assert!(x >= -size && x <= size);
        debug_assert!(y >= -size && y <= size);
        let center = SIZE / 2;
        // go up > decrease by self.width
        (center as i32 + (-y) * (self.width as i32) + x) as usize
    }
}

impl<T, const SIZE: usize> Index<(i32, i32)> for ConstCenteredArray<T, SIZE> {
    type Output = T;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        &self.arr[self.get_idx(index.0, index.1)]
    }
}

impl<T, const SIZE: usize> IndexMut<(i32, i32)> for ConstCenteredArray<T, SIZE> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        &mut self.arr[self.get_idx(index.0, index.1)]
    }
}


#[cfg(test)]
mod test {
    use crate::client::pathfind::moves::cenetered_arr::CenteredArray;

    #[test]
    fn test_values() {
        let mut arr = CenteredArray::init::<(i32, i32), 4>();
        for x in -4..=4 {
            for y in -4..=4 {
                arr[(x, y)] = (x, y);
            }
        }


        for x in -4..=4 {
            for y in -4..=4 {
                assert_eq!(arr[(x, y)], (x, y));
            }
        }
    }
}
