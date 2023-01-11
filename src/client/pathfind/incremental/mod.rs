#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use std::{
    collections::{BinaryHeap, HashMap},
    fmt::{Debug, Formatter},
    hash::Hash,
    time::Instant,
};

use crate::client::{
    pathfind::{
        traits::{GoalCheck, Heuristic, Progression, Progressor},
        MinHeapNode,
    },
    timing::Increment,
};

/// credit baritone
const COEFFICIENTS: [f64; 7] = [1.5, 2.0, 2.5, 3., 4., 5., 10.];
const MIN_DIST: f64 = 5.0;

/// An A-star Node
pub trait Node: Clone {
    /// # Purpose
    /// Sometimes nodes are very memory expensive. To reduce this only open
    /// nodes need to contain full state. Records can instead store a hash
    /// to show equality. Records should contain all the information
    /// needed in the path returned by A-star.
    ///
    /// # Equality
    /// If a Node is equal to another node its records should be equal and if
    /// nodes are not equal the records should not be equal.
    ///
    /// # Example
    /// If A-star is done on a block mining problem we need to store all blocks
    /// mined for every node. This is expensive as we only need to progress
    /// from the open set and in the path returned we will only need to record
    /// each block mined at an individual node. When progressing the parents
    /// of each node we can get the total state, but this is expensive so in the
    /// open set we will probably want to have some type of [`HashSet`] or
    /// [`HashMap`].
    ///
    /// ```
    /// For any node pair (Node_a, Node_b)
    /// and any records (Record_a, Record_b)
    /// Node_a == Node_b => Record_a == Record_b
    /// Node_a != Node_b => Record_a != Record_b
    /// ```
    type Record: PartialEq + Hash + Eq + Clone;

    /// Takes the node and turns it into a record see [`Self::Record`]
    fn get_record(&self) -> Self::Record;
}

pub struct AStar<T: Node> {
    state: Option<AStarState<T>>,
}

/// The state of `AStar`. This is a separate object so that when the iteration
/// is done the state can be moved
///
/// TODO: what?????
struct AStarState<T: Node> {
    /// given an idx return a record
    idx_to_record: Vec<T::Record>,

    /// given a record return an idx
    record_to_idx: HashMap<T::Record, usize>,

    /// tracks ancestors of records to reconstruct the final path
    /// `Vec<T::Record>`
    parent_map: HashMap<usize, usize>,

    /// **map record_id -> g_score**.
    /// the g-scores of all open nodes. How long it took to travel to them
    g_scores: HashMap<usize, f64>,

    /// a priority queue of nodes sorted my lowest f-score
    open_set: BinaryHeap<MinHeapNode<T, f64>>,

    /// if A-star is valid
    valid: bool,

    /// the total amount of time we have spent on the problem
    total_duration_ms: u128,

    max_duration_ms: u128,

    meta_heuristics: [f64; 7],
    meta_heuristics_ids: [usize; 7],
}

/// Takes ownership of all nodes and returns a path ending at `goal_idx` which
/// will start at a starting idx determined by tracing `parent_map`
/// HashMap<idx,idx> until there is no parent (i.e., the root node). This is the
/// most efficient path, so there should be no circles assuming non-negative
/// weights.
fn reconstruct_path<T: Clone>(
    vec: &[T],
    goal_idx: usize,
    parent_map: &HashMap<usize, usize>,
) -> Vec<T> {
    let init_value = vec[goal_idx].clone();

    let mut res = vec![init_value];

    let mut on_idx = goal_idx;
    while let Some(&next_idx) = parent_map.get(&on_idx) {
        res.push(vec[next_idx].clone());
        on_idx = next_idx;
    }

    // we did this in reverse order, we need to reverse the array
    res.reverse();
    res
}

pub struct PathResult<T> {
    pub complete: bool,
    pub value: Vec<T>,
}

impl<T: Debug> Debug for PathResult<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "PathResult{{complete: {:?}, value: {:?}}}",
            self.complete, self.value
        ))
    }
}

impl<T> PathResult<T> {
    fn complete(value: Vec<T>) -> Self {
        Self {
            complete: true,
            value,
        }
    }

    fn incomplete(value: Vec<T>) -> Self {
        Self {
            complete: false,
            value,
        }
    }
}

impl<T: Node> AStar<T> {
    pub fn new(init_node: T) -> Self {
        let init_record = init_node.get_record();

        let mut record_to_idx = HashMap::new();
        record_to_idx.insert(init_record.clone(), 0);

        let mut g_scores = HashMap::new();
        g_scores.insert(0, 0.0);

        let mut open_set = BinaryHeap::new();

        open_set.push(MinHeapNode {
            contents: init_node,
            score: f64::MAX,
        });

        let state = Some(AStarState {
            idx_to_record: vec![init_record],
            meta_heuristics: [f64::MAX; 7],
            record_to_idx,
            g_scores,
            open_set,
            total_duration_ms: 0,
            parent_map: nbt::Map::default(),
            valid: false,
            meta_heuristics_ids: [0; 7],
            max_duration_ms: 5000,
        });

        Self { state }
    }

    pub fn set_max_millis(&mut self, value: u128) {
        self.state.as_mut().unwrap().max_duration_ms = value;
    }

    pub fn select_best(&mut self) -> Increment<PathResult<T::Record>> {
        let state = self.state.take().unwrap();
        let mut best = (f64::MAX, 0);
        for i in 0..7 {
            let heuristic = state.meta_heuristics[i];
            let id = state.meta_heuristics_ids[i];
            if heuristic < best.0 {
                best = (heuristic, id);
            }
            let g_score = state.g_scores[&id];
            if g_score > MIN_DIST {
                println!("larger than min dist");
                let path = reconstruct_path(&state.idx_to_record, id, &state.parent_map);
                return Increment::Finished(PathResult::incomplete(path));
            }
        }
        let path = reconstruct_path(&state.idx_to_record, best.1, &state.parent_map);
        Increment::Finished(PathResult::incomplete(path))
    }

    pub fn iterate_until(
        &mut self,
        end_at: Instant,
        heuristic: &impl Heuristic<T>,
        progressor: &impl Progressor<T>,
        goal_check: &impl GoalCheck<T>,
    ) -> Increment<PathResult<T::Record>> {
        let iter_start = Instant::now();

        loop {
            let now = Instant::now();

            if now >= end_at {
                let iter_duration = now.duration_since(iter_start);
                let state = self.state.as_mut().unwrap();
                let dur = &mut state.total_duration_ms;
                *dur += iter_duration.as_millis();
                return if *dur > state.max_duration_ms {
                    println!("reached maxed duration");
                    return self.select_best();
                } else {
                    Increment::InProgress
                };
            }

            match self.iterate(heuristic, progressor, goal_check) {
                Increment::Finished(res) => {
                    return Increment::Finished(res);
                }
                Increment::InProgress => {}
            }
        }
    }
    pub fn iterate(
        &mut self,
        heuristic: &impl Heuristic<T>,
        progressor: &impl Progressor<T>,
        goal_check: &impl GoalCheck<T>,
    ) -> Increment<PathResult<T::Record>> {
        // obtain the state. If we have already finished the state is Option as we did
        // Option#take(..). We should not ever call this in that state.
        let Some(state) = self.state.as_mut() else { panic!("called after finished") };

        if let Some(node) = state.open_set.pop() {
            let parent = node.contents;

            // we have found the goal. Let's stop and return the reconstructed path
            if goal_check.is_goal(&parent) {
                let record = parent.get_record();
                let record_idx = state.record_to_idx[&record];
                let state = self.state.take().unwrap();
                let path = reconstruct_path(&state.idx_to_record, record_idx, &state.parent_map);

                return Increment::Finished(PathResult::complete(path));
            }

            let Progression::Movements(neighbors) = progressor.progressions(&parent) else { return Increment::InProgress };

            let parent_record = parent.get_record();
            let parent_record_idx = state.record_to_idx[&parent_record];

            let parent_g_score = *state.g_scores.get(&parent_record_idx).unwrap();

            'neighbor_loop: for neighbor in neighbors {
                let tentative_g_score = parent_g_score + neighbor.cost;

                let value = neighbor.value.clone();
                let record = value.get_record();

                let (record_idx, _g_score) = if let Some(idx) = state.record_to_idx.get(&record) {
                    let prev_g_score = state.g_scores.get_mut(idx).unwrap();
                    if tentative_g_score < *prev_g_score {
                        *prev_g_score = tentative_g_score;
                    } else {
                        continue 'neighbor_loop;
                    }
                    (*idx, tentative_g_score)
                } else {
                    let value_idx = state.idx_to_record.len();
                    state.idx_to_record.push(record.clone());
                    state.record_to_idx.insert(record, value_idx);
                    state.g_scores.insert(value_idx, tentative_g_score);
                    (value_idx, tentative_g_score)
                };

                state.parent_map.insert(record_idx, parent_record_idx);

                let h_score = heuristic.heuristic(&neighbor.value);
                let f_score = tentative_g_score + h_score;

                #[allow(clippy::needless_range_loop)]
                for i in 0..state.meta_heuristics.len() {
                    let meta_heuristic = h_score + tentative_g_score / COEFFICIENTS[i];
                    let current = state.meta_heuristics[i];
                    if meta_heuristic < current {
                        state.meta_heuristics[i] = meta_heuristic;
                        state.meta_heuristics_ids[i] = record_idx;
                        if !state.valid && tentative_g_score > MIN_DIST {
                            state.valid = true;
                        }
                    }
                }

                let heap_node = MinHeapNode {
                    contents: value,
                    score: f_score,
                };

                state.open_set.push(heap_node);
            }
        } else {
            println!(
                "no more nodes iterated through {}",
                state.idx_to_record.len()
            );
            return self.select_best();
        }

        Increment::InProgress
    }
}
