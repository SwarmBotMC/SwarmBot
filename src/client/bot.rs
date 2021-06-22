
use std::time::Duration;




use crate::client::follow::Follower;
use crate::client::pathfind::context::{GlobalContext};

use crate::client::pathfind::progress_checker::{NoVehicleProgressor};
use crate::client::timing::Increment;
use crate::protocol::{InterfaceOut, EventQueue};



use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;



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

        // always jump
        self.state.physics.jump();
        self.state.physics.tick(&global.world_blocks);

        self.out.teleport(self.state.physics.location());
        self.move_around();
        self.state.ticks += 1;
    }

    fn move_around(&mut self) {
        if self.state.ticks % 5 != 0 {
            return;
        }
        if let Some(mut follower) = self.state.follower.take() {
            follower.follow(&mut self.state, &mut self.out);
            self.state.follower = Some(follower);
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

        let res = traverse.a_star.iterate_for(Duration::from_millis(30), &traverse.heuristic, &progressor, &traverse.goal_checker);

        if let Increment::Finished(res) = res {
            if let Some(res) = res {
                println!("found goal of size {}", res.len());
                client.follower = Follower::new(res);
            } else {
                println!("could not find goal");
            }
            // we are done finding the path
            client.travel_problem = None;
        }
    }
}
