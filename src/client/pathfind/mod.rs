use std::cmp::Ordering;

pub mod traits;
pub mod bidirectional;
pub mod moves;
pub mod implementations;
pub mod context;
pub mod incremental;

pub struct MinHeapNode<T, C: PartialOrd + PartialEq> {
    pub contents: T,
    pub score: C,
}

impl <T: Clone, C: Clone + PartialOrd + PartialEq> Clone for MinHeapNode<T, C> {
    fn clone(&self) -> Self {
        MinHeapNode {
            contents: self.contents.clone(),
            score: self.score.clone()
        }
    }
}

impl <T, C: PartialOrd + PartialEq> Ord for MinHeapNode<T, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl <T,C :PartialOrd + PartialEq> PartialOrd for MinHeapNode<T, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl <T, C: PartialOrd + PartialEq> PartialEq for MinHeapNode<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}

impl <T, C: PartialOrd + PartialEq> Eq for MinHeapNode<T, C> {

}

pub struct Path<T> {
    pub inner: Vec<T>,
}
