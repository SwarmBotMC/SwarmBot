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
use crate::storage::block::{BlockKind, BlockLocation};
use crate::client::physics::tools::{Tool, Material};

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


    pub(crate) fn process_command(&mut self, name: &str, args: &[&str], global: &mut GlobalState){

        // println! but bold
        macro_rules! msg {
            () => {{
                println!();
            }};
            ($($msg: expr),*) => {{
                let to_print_raw = format!($($msg),*);
                let to_print = ansi_term::Color::Black.bold().paint(to_print_raw);
                println!("{}", to_print);
            }};
        }

        match name {
            "jump" => {
                self.state.physics.jump();
            }
            "follow" => {
                self.state.follow_closest = true;
            }
            "kys" => {

                // self.
                // let closest = self.global.world_blocks.closest(loc,|state| state.kind() == kind);
                // if
            }
            "goto" => {

                if let [id] = args {
                    let id: u32 = id.parse().unwrap();
                    let kind = BlockKind::from(id);

                    let loc = BlockLocation::from(self.state.physics.location());

                    let closest = global.world_blocks.closest(loc,|state| state.kind() == kind);

                    if let Some(closest) = closest {
                        self.state.travel_to_block(closest);
                    } else {
                        msg!("There is no block {} by me", id);
                    }

                }

                if let [a, b, c] = args {
                    let x = a.parse().unwrap();
                    let y = b.parse().unwrap();
                    let z = c.parse().unwrap();
                    let dest = BlockLocation::new(x,y,z);
                    self.state.travel_to_block(dest);
                }
            }
            "stop" => {
                self.state.travel_problem = None;
                self.state.last_problem = None;
            }
            "loc" => {
                msg!("My location is {}", self.state.physics.location());
            }
            "state" => {
                if let [name] = args {
                    if name == &self.state.info.username {
                        msg!("follower {:?}", self.state.follower);
                        msg!("location {}", self.state.physics.location());
                        msg!();
                    }
                }
            }
            "get" => {
                if let [a, b, c] = args {
                    let x = a.parse().unwrap();
                    let y = b.parse().unwrap();
                    let z = c.parse().unwrap();
                    let location = BlockLocation::new(x,y,z);

                    msg!("The block is {:?}", global.world_blocks.get_block(location));
                }
            }
            "mine" => {
                if let [id] = args {
                    let id: u32 = id.parse().unwrap();
                    let kind = BlockKind::from(id);

                    let origin = BlockLocation::from(self.state.physics.location());
                    let closest = global.world_blocks.closest(origin,|state| state.kind() == kind);

                    if let Some(closest) = closest {
                        let dir = closest.center_bottom() - origin.center_bottom();
                        self.state.physics.look(dir.into());

                        let tool = Tool::new(Material::DIAMOND);
                        let ticks = tool.wait_time(kind, false, true , &global.block_data);

                        msg!("started mining at {} .. ticks {}", closest, ticks);

                        let task = MineTask {
                            ticks,
                            location: closest
                        };

                        self.state.mining = Some(task);
                        self.out.mine(closest, Mine::Start);
                    }
                }
            }
            _ => {
                self.out.send_chat("invalid command");
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
