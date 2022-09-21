pub enum Increment<T> {
    Finished(T),
    InProgress,
}

/// Tasks which can be done incrementally. Used in pathfinding
pub trait Incremental<T> {
    /// complete an iteration. Returns Some(T) when it is finished
    fn iterate(&mut self) -> Increment<T>;
}
