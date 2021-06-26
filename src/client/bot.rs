use std::time::{Duration, Instant};

use itertools::min;

use crate::client::follow::{Follower, FollowResult};
use crate::client::pathfind::context::{GlobalContext, MoveNode};
use crate::client::pathfind::implementations::Problem;
use crate::client::state::global::GlobalState;
use crate::client::state::local::{LocalState, MineTask};
use crate::client::timing::Increment;
use crate::protocol::{EventQueue, InterfaceOut, Mine};
use float_ord::FloatOrd;
use crate::types::Direction;
use crate::client::physics::Line;
use crate::client::physics::speed::Speed;

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
        match self.state.mining.as_mut() {
            None => {}
            Some(mining) => {
                mining.ticks -= 1;
                self.out.left_click();
                if mining.ticks == 0 {
                    println!("finished mining");
                    self.out.mine(mining.location, Mine::Finished);
                    self.state.mining = None;
                }
            }
        }
        self.move_around(global);

        self.state.physics.tick(&global.world_blocks);

        let physics = &self.state.physics;
        self.out.teleport_and_look(physics.location(), physics.direction(), physics.on_ground());

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
        } else if self.state.follow_closest {
            let current_loc = self.state.physics.location();
            let closest = global.world_entities.iter().min_by_key(|(id, data)| {
                FloatOrd(data.location.dist2(current_loc))
            });

            if let Some((id, data)) = closest {
                let displacement = data.location - current_loc;
                if displacement.has_length() {
                    let dir = Direction::from(displacement);
                    self.state.physics.look(dir);
                    self.state.physics.line(Line::Forward);
                    self.state.physics.speed(Speed::WALK);
                }
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
