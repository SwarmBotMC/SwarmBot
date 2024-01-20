use std::cmp::Ordering;

pub mod context;
pub mod implementations;
pub mod incremental;
pub mod moves;
pub mod traits;

/// a heap node sorted-min
pub struct MinHeapNode<T, C: PartialOrd + PartialEq> {
    /// the contents of the node
    pub contents: T,
    /// the score by which to sort
    pub score: C,
}

impl<T, C: PartialOrd + PartialEq> MinHeapNode<T, C> {
    /// Create a new [`MinHeapNode`]
    #[allow(unused)]
    pub const fn new(contents: T, score: C) -> Self {
        Self { contents, score }
    }
}

impl<T: Clone, C: Clone + PartialOrd + PartialEq> Clone for MinHeapNode<T, C> {
    fn clone(&self) -> Self {
        Self {
            contents: self.contents.clone(),
            score: self.score.clone(),
        }
    }
}

impl<T, C: PartialOrd + PartialEq> Ord for MinHeapNode<T, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.partial_cmp(&self.score).unwrap()
    }
}

impl<T, C: PartialOrd + PartialEq> PartialOrd for MinHeapNode<T, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, C: PartialOrd + PartialEq> PartialEq for MinHeapNode<T, C> {
    #[allow(clippy::unconditional_recursion)]
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}

impl<T, C: PartialOrd + PartialEq> Eq for MinHeapNode<T, C> {}
