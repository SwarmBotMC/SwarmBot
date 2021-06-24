use std::time::Duration;
use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::{GlobalContext, MoveContext};
use crate::client::pathfind::progress_checker::NoVehicleProgressor;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::client::timing::Increment;
use crate::protocol::{EventQueue, InterfaceOut};
use crate::client::physics::{Walk};

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
                    problem.recalc(MoveContext::no_blocks(block_loc));
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


pub fn run_threaded(client: &mut LocalState, global: &GlobalState) {
    if let Some(traverse) = client.travel_problem.as_mut() {
        let ctx = GlobalContext {
            path_config: &global.travel_config,
            world: &global.world_blocks,
        };

        let progressor = NoVehicleProgressor::new(ctx);

        let res = traverse.iterate_for(Duration::from_millis(30), &progressor);

        if let Increment::Finished(res) = res {
            println!("found goal of size {} .. complete = {}", res.value.len(), res.complete);
            client.follower = Follower::new(res);

            // we are done finding the path
            client.last_problem = client.travel_problem.take();
        }
    }
}
