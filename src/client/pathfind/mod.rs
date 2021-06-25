use std::cmp::Ordering;

pub mod traits;
pub mod bidirectional;
pub mod moves;
pub mod implementations;
pub mod context;
pub mod incremental;

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
