use std::cmp::Ordering;

pub mod progress_checker;
pub mod world_blocks;
pub mod pathfinding;
pub mod bidirectional;
pub mod moves;
mod context;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct BlockLocation(pub i64, pub i64, pub i64);

impl BlockLocation {
    fn dist2(&self, other: BlockLocation) -> i64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        let dz = self.2 - other.2;
        dx*dx + dy*dy + dz*dz
    }

    fn dist(&self, other: BlockLocation) -> f64 {
        (self.dist2(other) as f64).sqrt()
    }
}

pub struct HeapNode<T> {
    pub contents: T,
    pub f_score: f64,
}

impl <T: Clone> Clone for HeapNode<T> {
    fn clone(&self) -> Self {
        HeapNode {
            contents: self.contents.clone(),
            f_score: self.f_score
        }
    }
}

impl <T> Ord for HeapNode<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl <T> PartialOrd for HeapNode<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.f_score.partial_cmp(&self.f_score)
    }
}

impl <T> PartialEq for HeapNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.f_score.eq(&other.f_score)
    }
}

impl <T> Eq for HeapNode<T> {

}

pub struct Path<T> {
    pub inner: Vec<T>,
}
