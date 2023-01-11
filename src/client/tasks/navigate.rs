use std::time::Instant;

use interfaces::types::{BlockLocation, ChunkLocation};

use crate::{
    client::{
        follow::{Follower, Result},
        pathfind::{
            context::MoveNode,
            implementations::{
                no_vehicle::{
                    BlockGoalCheck, BlockHeuristic, BlockNearGoalCheck, CenterChunkGoalCheck,
                    ChunkHeuristic, TravelProblem,
                },
                PlayerProblem, Problem,
            },
            traits::{GoalCheck, Heuristic},
        },
        state::{global::GlobalState, local::LocalState},
        tasks::TaskTrait,
        timing::Increment,
    },
    protocol::InterfaceOut,
};

pub type ChunkTravelTask = NavigateProblem<ChunkHeuristic, CenterChunkGoalCheck>;
pub type BlockTravelTask = NavigateProblem<BlockHeuristic, BlockGoalCheck>;
pub type BlockTravelNearTask = NavigateProblem<BlockHeuristic, BlockNearGoalCheck>;

impl ChunkTravelTask {
    #[allow(unused)]
    pub fn new(goal: ChunkLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_center_chunk(start, goal);
        problem.into()
    }
}

impl BlockTravelTask {
    pub fn new(goal: BlockLocation, local: &LocalState) -> Self {
        let start = local.physics.location().into();
        let problem = TravelProblem::navigate_block(start, goal);
        problem.into()
    }
}

pub struct NavigateProblem<H: Heuristic, G: GoalCheck> {
    calculate: bool,
    problem: Box<PlayerProblem<H, G>>,
    follower: Option<Follower>,
}

impl<H: Heuristic, G: GoalCheck> From<PlayerProblem<H, G>> for NavigateProblem<H, G> {
    fn from(problem: PlayerProblem<H, G>) -> Self {
        Self {
            calculate: true,
            problem: box problem,
            follower: None,
        }
    }
}

impl<H: Heuristic + Send + Sync, G: GoalCheck + Send + Sync> TaskTrait for NavigateProblem<H, G> {
    fn tick(
        &mut self,
        _out: &mut impl InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> bool {
        let Some(follower) = self.follower.as_mut() else { return false };

        if follower.should_recalc() {
            println!("recalc");
            self.problem
                .recalc(MoveNode::simple(local.physics.location().into()));
            self.calculate = true;
        }

        match follower.follow_iteration(local, global) {
            Result::Failed => {
                println!("failed");
                self.follower = None;
                self.problem
                    .recalc(MoveNode::simple(local.physics.location().into()));
                self.calculate = true;
                false
            }
            Result::InProgress => false,
            Result::Finished => {
                println!("finished!");
                true
            }
        }
    }

    fn expensive(&mut self, end_at: Instant, local: &mut LocalState, global: &GlobalState) {
        if !self.calculate {
            return;
        }

        let res = self.problem.iterate_until(end_at, local, global);
        match res {
            Increment::Finished(res) => {
                self.calculate = false;
                match self.follower.as_mut() {
                    None => self.follower = Follower::new(res),
                    Some(before) => before.merge(res),
                };
            }

            // Nothing as we are still in progress
            Increment::InProgress => {}
        }
    }
}
