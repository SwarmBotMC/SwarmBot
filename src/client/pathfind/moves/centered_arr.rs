use std::ops::{Index, IndexMut};

pub struct CenteredArray<T> {
    arr: Vec<T>,
    width: usize,
    size: usize,
}

impl<T: Default + Copy> CenteredArray<T> {
    pub fn init(r: usize) -> Self {
        let size = (2 * r + 1) * (2 * r + 1);
        let width: usize = 2 * r + 1;
        let arr = vec![T::default(); size];
        Self { arr, width, size }
    }
}

impl<T> CenteredArray<T> {
    fn get_idx(&self, x: i32, y: i32) -> usize {
        let size = self.size as i32;
        debug_assert!(x >= -size && x <= size);
        debug_assert!(y >= -size && y <= size);
        let center = self.size / 2;
        // go up > decrease by self.width
        (center as i32 + (-y) * (self.width as i32) + x) as usize
    }
}

impl<T> Index<(i32, i32)> for CenteredArray<T> {
    type Output = T;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        &self.arr[self.get_idx(index.0, index.1)]
    }
}

impl<T> IndexMut<(i32, i32)> for CenteredArray<T> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        let idx = self.get_idx(index.0, index.1);
        &mut self.arr[idx]
    }
}

#[cfg(test)]
mod test {
    use crate::client::pathfind::moves::centered_arr::CenteredArray;

    #[test]
    fn test_values() {
        let mut arr = CenteredArray::init(4);
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
