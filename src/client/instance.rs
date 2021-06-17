use packets::types::UUID;

use crate::client::runner::GlobalState;
use crate::protocol::McProtocol;
use crate::storage::world::WorldBlocks;
use crate::types::Location;
use crate::storage::block::BlockLocation;
// use crate::client::pathfind::bidirectional::a_star_bi;
use crate::client::pathfind::progress_checker::{NoVehicleProgressor, NoVehicleHeuristic, NoVehicleGoalCheck, GoalCheck};
use crate::client::pathfind::context::{GlobalContext, MoveContext};
use crate::client::pathfind::incremental::{AStar, PathResult};
use std::time::Duration;
use crate::client::timing::Increment;
use crate::client::follow::Follower;

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}


pub struct State {
    pub ticks: usize,
    pub info: ClientInfo,
    pub destination: BlockLocation,
    pub alive: bool,
    pub follower: Option<Follower>,
    pub finder_problem: Option<TravelProblem>,
    pub location: Location
}

pub struct TravelPath {
    blocks: Vec<BlockLocation>
}

pub struct TravelProblem {
    a_star: AStar<MoveContext>,
    heuristic: NoVehicleHeuristic,
    goal_checker: NoVehicleGoalCheck
}

pub struct Client<T: McProtocol> {
    pub state: State,
    pub protocol: T,
}

const fn ticks_from_secs(seconds: usize) -> usize {
    seconds * 20
}

impl<T: McProtocol> Client<T> {
    pub fn run_sync(&mut self, global: &mut GlobalState) {
        self.move_around();
        self.anti_afk();

        self.state.ticks += 1;
    }

    fn anti_afk(&mut self) {
        const MESSAGE_TICKS: usize = ticks_from_secs(15); // every 15 seconds
        if self.state.ticks % MESSAGE_TICKS == 0 {
            // throwaway command to prevent anti afk
            self.protocol.send_chat("/wot");
        }
    }

    fn move_around(&mut self){
        if self.state.ticks % 20 != 0 {
            return;
        }
        if let Some(mut follower) = self.state.follower.take() {
            follower.follow(&mut self.state, &mut self.protocol);
            self.state.follower = Some(follower);
        }
    }
}



pub fn run_threaded(client: &mut State, global: &GlobalState){

    if let Some(traverse) = client.finder_problem.as_mut() {

        let ctx = GlobalContext {
            path_config: &global.travel_config,
            world: &global.world_blocks
        };

        let progressor = NoVehicleProgressor::new(ctx);

        let res = traverse.a_star.iterate_for(Duration::from_millis(30), &traverse.heuristic, &progressor, &traverse.goal_checker);

        if let Increment::Finished(res) = res {

            if let Some(res) = res {
                client.follower = Follower::new(res);
            }
            // we are done finding the path
            client.finder_problem = None;
        }
    }
}