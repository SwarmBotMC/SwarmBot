use interfaces::types::{BlockLocation, BlockLocation2D, ChunkLocation};

use crate::client::pathfind::{
    context::MoveNode,
    implementations::PlayerProblem,
    traits::{GoalCheck, Heuristic},
};

pub struct BlockGoalCheck {
    goal: BlockLocation,
}

pub struct BlockNearGoalCheck {
    goal: BlockLocation2D,
    dist2: f64,
    must_not_hit: bool,
}

impl BlockNearGoalCheck {
    const fn new(goal: BlockLocation2D, dist2: f64, must_not_hit: bool) -> Self {
        Self {
            goal,
            dist2,
            must_not_hit,
        }
    }
}

impl GoalCheck for BlockNearGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        let input = BlockLocation2D::from(input.location);
        let dist2 = input.dist2(self.goal) as f64;
        let same = if self.must_not_hit {
            self.goal.x == input.x && self.goal.z == input.z
        } else {
            false
        };
        !same && dist2 <= self.dist2
    }
}

pub struct ChunkGoalCheck {
    goal: ChunkLocation,
}

impl GoalCheck for ChunkGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        let cx = input.location.x >> 4;
        let cz = input.location.z >> 4;
        let chunk_loc = ChunkLocation(cx, cz);
        chunk_loc == self.goal
    }
}

pub struct CenterChunkGoalCheck {
    goal_center_x: i32,
    goal_center_z: i32,
}

impl CenterChunkGoalCheck {
    const fn new(goal: ChunkLocation) -> Self {
        let goal_center_x = (goal.0 << 4) + 8;
        let goal_center_z = (goal.1 << 4) + 8;
        Self {
            goal_center_x,
            goal_center_z,
        }
    }
}

impl GoalCheck for CenterChunkGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        let dx = self.goal_center_x - input.location.x;
        let dz = self.goal_center_z - input.location.z;

        (0..=1).contains(&dx) && (0..=1).contains(&dz)
    }
}

impl BlockGoalCheck {
    pub const fn new(goal: BlockLocation) -> Self {
        Self { goal }
    }
}

impl GoalCheck for BlockGoalCheck {
    fn is_goal(&self, input: &MoveNode) -> bool {
        let close_y = (input.location.y - self.goal.y).abs() <= 1;
        close_y && input.location.x == self.goal.x && input.location.z == self.goal.z
    }
}

pub struct BlockHeuristic {
    pub move_cost: f64,
    pub goal: BlockLocation,
}

impl Heuristic for BlockHeuristic {
    fn heuristic(&self, input: &MoveNode) -> f64 {
        let current = input.location;
        current.dist(self.goal) * self.move_cost * 0.2
    }
}

#[derive(Debug)]
pub struct ChunkHeuristic {
    pub move_cost: f64,
    pub center_x: i32,
    pub center_z: i32,
}

impl ChunkHeuristic {
    const fn new(goal: ChunkLocation, move_cost: f64) -> Self {
        Self {
            move_cost,

            // the center of the chunk
            center_x: (goal.0 << 4) + 8,
            center_z: (goal.1 << 4) + 8,
        }
    }
}

impl Heuristic for ChunkHeuristic {
    fn heuristic(&self, input: &MoveNode) -> f64 {
        let dx = f64::from(input.location.x - self.center_x);
        let dz = f64::from(input.location.z - self.center_z);
        let dist2 = dx.mul_add(dx, dz * dz);
        dist2.sqrt() * self.move_cost * 0.2
    }
}

pub struct TravelProblem;

pub type TravelNearProblem = PlayerProblem<BlockHeuristic, BlockNearGoalCheck>;
pub type TravelBlockProblem = PlayerProblem<BlockHeuristic, BlockGoalCheck>;
pub type TravelChunkProblem = PlayerProblem<ChunkHeuristic, ChunkGoalCheck>;
pub type TravelChunkCenterProblem = PlayerProblem<ChunkHeuristic, CenterChunkGoalCheck>;

impl TravelProblem {
    pub fn navigate_block(start: BlockLocation, goal: BlockLocation) -> TravelBlockProblem {
        let heuristic = BlockHeuristic {
            move_cost: 1.0,
            goal,
        };
        let start_node = MoveNode::simple(start);
        let goal_checker = BlockGoalCheck::new(goal);
        PlayerProblem::new(start_node, heuristic, goal_checker)
    }

    pub fn navigate_near_block(
        start: BlockLocation,
        goal: BlockLocation2D,
        dist2: f64,
        must_not_hit: bool,
    ) -> TravelNearProblem {
        let heuristic = BlockHeuristic {
            move_cost: 1.0,
            goal: goal.into(),
        };
        let start_node = MoveNode::simple(start);
        let goal_checker = BlockNearGoalCheck::new(goal, dist2, must_not_hit);
        PlayerProblem::new(start_node, heuristic, goal_checker)
    }

    #[allow(unused)]
    pub fn navigate_chunk(start: BlockLocation, goal: ChunkLocation) -> TravelChunkProblem {
        let heuristic = ChunkHeuristic::new(goal, 1.0);
        let start_node = MoveNode::simple(start);
        let goal_checker = ChunkGoalCheck { goal };
        PlayerProblem::new(start_node, heuristic, goal_checker)
    }

    #[allow(unused)]
    pub fn navigate_center_chunk(
        start: BlockLocation,
        goal: ChunkLocation,
    ) -> TravelChunkCenterProblem {
        let heuristic = ChunkHeuristic::new(goal, 1.0);
        let start_node = MoveNode::simple(start);
        let goal_checker = CenterChunkGoalCheck::new(goal);
        PlayerProblem::new(start_node, heuristic, goal_checker)
    }
}
