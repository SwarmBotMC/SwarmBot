use std::rc::Rc;
use std::time::Duration;

use packets::types::UUID;
use tokio::sync::Notify;

use crate::client::follow::Follower;
use crate::client::pathfind::context::{GlobalContext, MoveContext};
use crate::client::pathfind::incremental::{AStar, PathResult};
use crate::client::pathfind::progress_checker::{GoalCheck, NoVehicleGoalCheck, NoVehicleHeuristic, NoVehicleProgressor};
use crate::client::timing::Increment;
use crate::protocol::McProtocol;
use crate::storage::block::BlockLocation;
use crate::storage::world::WorldBlocks;
use crate::types::Location;
use crate::client::state::local::LocalState;
use crate::client::state::global::GlobalState;


#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}

pub struct TravelPath {
    blocks: Vec<BlockLocation>,
}

pub struct TravelProblem {
    pub a_star: AStar<MoveContext>,
    pub heuristic: NoVehicleHeuristic,
    pub goal_checker: NoVehicleGoalCheck,
    pub notifier: Rc<Notify>,
}

impl Drop for TravelProblem {
    fn drop(&mut self) {
        self.notifier.notify_one();
    }
}

pub struct Client<T: McProtocol> {
    pub state: LocalState,
    pub protocol: T,
}

const fn ticks_from_secs(seconds: usize) -> usize {
    seconds * 20
}

impl<T: McProtocol> Client<T> {
    pub fn run_sync(&mut self, global: &mut GlobalState) {
        self.move_around();
        self.state.ticks += 1;
    }

    fn move_around(&mut self) {
        if self.state.ticks % 20 != 0 {
            return;
        }
        if let Some(mut follower) = self.state.follower.take() {
            follower.follow(&mut self.state, &mut self.protocol);
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
                client.follower = Follower::new(res);
            }
            // we are done finding the path
            client.travel_problem = None;
        }
    }
}
