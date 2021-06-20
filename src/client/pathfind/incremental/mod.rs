use std::collections::{HashMap, BinaryHeap};
use std::hash::Hash;
use crate::client::pathfind::HeapNode;
use crate::client::timing::{Increment};
use crate::client::pathfind::progress_checker::{Heuristic, Progressor, GoalCheck, Progression};
use std::time::{Duration, Instant};
use std::fmt::Debug;


pub struct AStar<T: Clone> {
    state: Option<AStarState<T>>
}


struct AStarState<T: Clone> {
    /// stores values of object T
    nodes: Vec<T>,
    val_to_idx: HashMap<T, usize>,

    /// **map id -> g_score**.
    /// the g-scores of all open nodes. How long it took to travel to them
    g_scores: HashMap<usize, f64>,

    /// The **open set**.
    /// a priority queue of nodes sorted my lowest f-score
    queue: BinaryHeap<HeapNode<usize>>,

    /// tracks ancestors of nodes to reconstruct the final path
    parent_map: HashMap<usize, usize>,
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

pub type PathResult<T> = Option<Vec<T>>;

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
            val_to_idx,
            g_scores,
            queue,
            parent_map: Default::default()
        });

        AStar {
            state
        }
    }

    pub fn iterate_for(&mut self, duration: Duration, heuristic: &impl Heuristic<T>, progressor: &impl Progressor<T>, goal_check: &impl GoalCheck<T>) -> Increment<PathResult<T>> {

        let start = Instant::now();

        loop {
            let on = Instant::now();

            let current_duration = on.duration_since(start);

            if current_duration >= duration {
                return Increment::InProgress;
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
                return Increment::Finished(PathResult::Some(path))
            }

            let neighbors = match progressor.progressions(popped) {
                Progression::Edge => {

                    // still return a path of on edge
                    let state = self.state.take().unwrap();
                    let path = reconstruct_path(state.nodes, idx, &state.parent_map);

                    return Increment::Finished(PathResult::Some(path));
                }
                Progression::Movements(neighbors) => {
                    neighbors
                }
            };

            // println!("neighbors {:?}", neighbors);

            let popped_g_score = *state.g_scores.get(&idx).unwrap();

            'neighbor_loop:
            for neighbor in neighbors {
                let g_score = popped_g_score + neighbor.cost;
                let value = neighbor.value.clone();

                let value_idx = state.val_to_idx.get(&value);

                let value_idx  = match value_idx {
                    Some(idx) => {
                        let prev_g_score = state.g_scores.get_mut(idx).unwrap();
                        if g_score < *prev_g_score {
                            *prev_g_score = g_score
                        } else {
                            continue 'neighbor_loop;
                        }
                        *idx
                    }
                    None => {
                        let value_idx = state.nodes.len();
                        state.nodes.push(value.clone());
                        state.val_to_idx.insert(value.clone(), value_idx);
                        state.g_scores.insert(value_idx, g_score);
                        value_idx
                    }
                };

                let h_score = heuristic.heuristic(&neighbor.value);
                let f_score = g_score + h_score;

                let heap_node = HeapNode {
                    contents: value_idx,
                    f_score,
                };

                state.queue.push(heap_node);

            }
        } else {
            return Increment::Finished(None)
        }

        Increment::InProgress
    }
}
