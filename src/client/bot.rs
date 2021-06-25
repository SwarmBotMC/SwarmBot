use std::time::{Duration, Instant};
use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::{GlobalContext, MoveNode};
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::protocol::{EventQueue, InterfaceOut};
use crate::client::pathfind::implementations::Problem;

type Prob = Box<dyn Problem<Node=MoveNode>>;

pub struct Bot<Queue: EventQueue, Out: InterfaceOut> {
    pub state: LocalState,
    pub queue: Queue,
    pub out: Out,
}

const fn ticks_from_secs(seconds: usize) -> usize {
    seconds * 20
}

impl<Queue: EventQueue, Out: InterfaceOut> Bot<Queue, Out> {
    pub fn run_sync(&mut self, global: &mut GlobalState) {
        self.move_around(global);

        self.state.physics.tick(&global.world_blocks);

        self.out.teleport(self.state.physics.location());
        self.out.look(self.state.physics.direction());

        self.state.ticks += 1;
    }

    fn move_around(&mut self, global: &mut GlobalState) {
        if let Some(mut follower) = self.state.follower.take() {
            let follow_result = follower.follow(&mut self.state, global);
            if follow_result == FollowResult::Failed || follower.should_recalc() {

                if let Some(mut problem) = self.state.last_problem.take() {
                    let block_loc = self.state.physics.location().into();
                    problem.recalc(MoveNode::simple(block_loc));
                    self.state.travel_problem = Some(problem);
                }

                if follow_result == FollowResult::Failed {
                    self.state.follower = None;
                }
            } else {
                self.state.follower = Some(follower);
            }

        }
    }
}

pub fn run_threaded(scope: &rayon::Scope, local: &mut LocalState, global: &GlobalState, end_by: Instant) {

    // TODO: this is pretty jank
    if let Some(mut traverse) = local.travel_problem.take() {

        let res = traverse.iterate_until(end_by, local, global);

        if let Increment::Finished(res) = res {
            println!("found goal of size {} .. complete = {}", res.value.len(), res.complete);
            local.follower = Follower::new(res);

            // we are done finding the path
            local.last_problem = Some(traverse);
        } else {
            local.travel_problem = Some(traverse)
        }

    }
}
