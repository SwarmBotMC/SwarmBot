use std::collections::{HashMap, BinaryHeap};
use std::hash::Hash;
use crate::client::pathfind::HeapNode;
use crate::client::timing::{Increment};
use crate::client::pathfind::progress_checker::{Heuristic, Progressor, GoalCheck, Progression};
use std::time::{Duration, Instant};
use std::fmt::Debug;


/// credit baritone
const COEFFICIENTS: [f64; 7] = [1.5, 2.0, 2.5, 3., 4., 5., 10.];
const MIN_DIST: f64 = 5.0;
const MIN_DIST2: f64 = MIN_DIST * MIN_DIST;

const MAX_DURATION_SECS: u64 = 5;

pub struct AStar<T: Clone> {
    state: Option<AStarState<T>>
}

/// The state of AStar. This is a separate object so that when the iteration is done the state can be moved
struct AStarState<T: Clone> {
    /// stores values of object T
    nodes: Vec<T>,
    val_to_idx: HashMap<T, usize>,

    /// **map id -> g_score**.
    /// the g-scores of all open nodes. How long it took to travel to them
    g_scores: HashMap<usize, f64>,

    meta_heuristics: [f64; 7],
    meta_heuristics_ids: [usize; 7],

    total_duration_s: u64,

    /// The **open set**.
    /// a priority queue of nodes sorted my lowest f-score
    queue: BinaryHeap<HeapNode<usize>>,

    /// tracks ancestors of nodes to reconstruct the final path
    parent_map: HashMap<usize, usize>,

    valid: bool
}

pub type Path<T> = Vec<T>;

/// Takes ownership of all nodes and returns a path ending at goal_idx which will start
/// at a starting idx determined by tracing parent_map HashMap<idx,idx> until there
/// is no parent (i.e., the root node). This is the most efficient path, so there should
/// be no circles assuming non-negative weights.
fn reconstruct_path<T: Clone>(vec: Vec<T>, goal_idx: usize, parent_map: &HashMap<usize, usize>) -> Vec<T> {

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

pub struct PathResult<T>{
    pub complete: bool,
    pub value: Vec<T>
}

impl <T> PathResult<T> {
    fn complete(value: Vec<T>) -> PathResult<T> {
        PathResult {
            complete: true,
            value
        }
    }

    fn incomplete(value: Vec<T>) -> PathResult<T> {
        PathResult {
            complete: false,
            value
        }
    }
}

impl <T: Clone + Hash + Eq + Debug> AStar<T> {

    pub fn new(init_node: T) -> AStar<T> {


        let mut val_to_idx = HashMap::new();
        val_to_idx.insert(init_node.clone(), 0);

        let mut g_scores = HashMap::new();
        g_scores.insert(0, 0.0);

        let mut queue = BinaryHeap::new();
        queue.push(HeapNode {
            contents: 0,
            f_score: f64::MAX
        });

        let state = Some(AStarState {
            nodes: vec![init_node],
            meta_heuristics: [f64::MAX; 7],
            val_to_idx,
            g_scores,
            queue,
            total_duration_s: 0,
            parent_map: Default::default(),
            valid: false,
            meta_heuristics_ids: [0; 7]
        });

        AStar {
            state
        }
    }

    pub fn select_best(&mut self, h: &impl Heuristic<T>) -> Increment<PathResult<T>> {
        let state = self.state.take().unwrap();
        let mut best = (f64::MAX, 0);
        for i in 0..7 {
            let heuristic = state.meta_heuristics[i];
            let id = state.meta_heuristics_ids[i];
            let value = state.nodes[id].clone();
            if heuristic < best.0 {
                best = (heuristic, id);
            }
            let g_score = state.g_scores[&id];
            if g_score > MIN_DIST {
                let h = h.heuristic(&value);
                let path = reconstruct_path(state.nodes, id, &state.parent_map);
                return Increment::Finished(PathResult::incomplete(path));
            }
        }
        let path = reconstruct_path(state.nodes, best.1, &state.parent_map);
        Increment::Finished(PathResult::incomplete(path))
    }

    pub fn iterate_for(&mut self, duration: Duration, heuristic: &impl Heuristic<T>, progressor: &impl Progressor<T>, goal_check: &impl GoalCheck<T>) -> Increment<PathResult<T>> {

        let start = Instant::now();

        loop {
            let on = Instant::now();

            let current_duration = on.duration_since(start);

            if current_duration >= duration {
                let dur = &mut self.state.as_mut().unwrap().total_duration_s;
                *dur += current_duration.as_secs();
                return if *dur > MAX_DURATION_SECS {
                    return self.select_best(heuristic);
                } else {
                    Increment::InProgress
                }
            }

            match self.iterate(heuristic, progressor, goal_check)  {
                Increment::Finished(res) => {
                    return Increment::Finished(res)
                }
                Increment::InProgress => {}
            }
        }
    }
    pub fn iterate(&mut self, heuristic: &impl Heuristic<T>, progressor: &impl Progressor<T>, goal_check: &impl GoalCheck<T>) -> Increment<PathResult<T>> {

        let state = match self.state.as_mut() {
            None => panic!("called after finished"),
            Some(state) => state
        };

        if let Some(node) = state.queue.pop() {

            let idx = node.contents;
            let popped = &state.nodes[idx];

            if goal_check.is_goal(popped) {
                let state = self.state.take().unwrap();
                let path = reconstruct_path(state.nodes, idx, &state.parent_map);
                return Increment::Finished(PathResult::complete(path));
            }

            let neighbors = match progressor.progressions(popped) {
                Progression::Movements(neighbors) => {
                    neighbors
                }
                _ => return Increment::InProgress,
            };

            let popped_g_score = *state.g_scores.get(&idx).unwrap();

            'neighbor_loop:
            for neighbor in neighbors {
                let tentative_g_score = popped_g_score + neighbor.cost;
                let value = neighbor.value.clone();

                let value_idx = state.val_to_idx.get(&value);

                let value_idx  = match value_idx {
                    Some(idx) => {
                        let prev_g_score = state.g_scores.get_mut(idx).unwrap();
                        if tentative_g_score < *prev_g_score {
                            *prev_g_score = tentative_g_score
                        } else {
                            continue 'neighbor_loop;
                        }
                        *idx
                    }
                    None => {
                        let value_idx = state.nodes.len();
                        state.nodes.push(value.clone());
                        state.val_to_idx.insert(value.clone(), value_idx);
                        state.g_scores.insert(value_idx, tentative_g_score);
                        value_idx
                    }
                };

                state.parent_map.insert(value_idx, idx);

                let h_score = heuristic.heuristic(&neighbor.value);
                let f_score = tentative_g_score + h_score;

                for i in 0..state.meta_heuristics.len() {

                    let meta_heuristic = h_score + tentative_g_score / COEFFICIENTS[i];
                    let current = state.meta_heuristics[i];
                    if meta_heuristic < current {
                        state.meta_heuristics[i] = meta_heuristic;
                        state.meta_heuristics_ids[i] = value_idx;
                        if !state.valid && tentative_g_score > MIN_DIST {
                            state.valid = true;
                        }
                    }

                }

                let heap_node = HeapNode {
                    contents: value_idx,
                    f_score,
                };

                state.queue.push(heap_node);

            }
        } else {
            println!("no more nodes .. expanded {}", state.nodes.len());
            return self.select_best(heuristic);
        }

        Increment::InProgress
    }
}
