use crate::client::state::global::GlobalState;
use crate::client::state::local::{LocalState, MineTask};
use crate::protocol::{InterfaceOut, Mine};
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::chunk::ChunkColumn;
use crate::storage::blocks::ChunkLocation;
use crate::types::{Chat, Location, LocationOrigin, Direction};
use crate::client::pathfind::traits::{Progressor, Progression};
use crate::client::pathfind::context::{GlobalContext, MoveNode};

pub trait InterfaceIn {
    fn on_chat(&mut self, message: Chat);
    fn on_death(&mut self);
    fn on_move(&mut self, location: Location);
    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn);
    fn on_entity_move(&mut self, id: u32, location: LocationOrigin);
    fn on_block_change(&mut self, location: BlockLocation, state: BlockState);
    fn on_entity_destroy(&mut self, id: u32);
    fn on_entity_spawn(&mut self, id: u32, location: Location);
    fn on_disconnect(&mut self, reason: &str);
    fn on_socket_close(&mut self);
}

pub struct SimpleInterfaceIn<'a, I: InterfaceOut> {
    global: &'a mut GlobalState,
    local: &'a mut LocalState,
    out: &'a mut I,
}

impl<I: InterfaceOut> SimpleInterfaceIn<'a, I> {
    pub fn new(local: &'a mut LocalState, global: &'a mut GlobalState, out: &'a mut I) -> SimpleInterfaceIn<'a, I> {
        SimpleInterfaceIn {
            local,
            global,
            out,
        }
    }
}

impl<'a, I: InterfaceOut> InterfaceIn for SimpleInterfaceIn<'a, I> {
    fn on_chat(&mut self, message: Chat) {
        if let Some(player_msg) = message.player_message() {
            if let Some(cmd) = player_msg.into_cmd() {
                match cmd.command {
                    "goto" => {

                        if let [id] = cmd.args[..] {
                            let id: u32 = id.parse().unwrap();

                            let loc = BlockLocation::from(self.local.physics.location());
                            let iter = self.global.world_blocks.select(loc,|state| state.id() == id)
                                .take(1);

                            for loc in iter {
                                self.out.send_chat(&format!("instance at {}", loc));
                                self.local.travel_to_block(loc);
                            }
                        }

                        if let [a, b, c] = cmd.args[..] {
                            let x = a.parse().unwrap();
                            let y = b.parse().unwrap();
                            let z = c.parse().unwrap();
                            let dest = BlockLocation::new(x,y,z);
                            self.local.travel_to_block(dest);
                        }
                    }
                    "loc" => {
                        let block_loc: BlockLocation = self.local.physics.location().into();
                        self.out.send_chat(&format!("my location is {}. My block loc is {}", self.local.physics.location(), block_loc));
                    }
                    "state" => {
                        if let [name] = cmd.args[..] {
                            if name == self.local.info.username {
                                println!("follower {:?}", self.local.follower);
                                println!("location {}", self.local.physics.location());
                                println!();
                            }
                        }
                    }
                    "mine" => {
                        if let [id] = cmd.args[..] {
                            let id: u32 = id.parse().unwrap();

                            let origin = BlockLocation::from(self.local.physics.location());
                            let closest = self.global.world_blocks.select(origin,|state| state.id() == id)
                                .take(50)
                                .min_by_key(|loc| loc.dist2(origin));

                            if let Some(closest) = closest {
                                println!("started mining at {}", closest);
                                let dir = closest.centered() - origin.centered();
                                self.local.physics.look(dir.into());

                                let task = MineTask {
                                    ticks: 20,
                                    location: closest
                                };

                                self.local.mining = Some(task);
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
    }

    fn on_death(&mut self) {
        self.out.respawn();
        self.out.send_chat("I died... oof... well I guess I should respawn");
    }

    fn on_move(&mut self, location: Location) {
        self.local.physics.teleport(location);
    }

    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn) {
        self.global.world_blocks.add_column(location, column);
    }

    fn on_entity_move(&mut self, id: u32, location: LocationOrigin) {
        self.global.world_entities.update_entity(id, self.local.bot_id, location);
    }

    fn on_block_change(&mut self, location: BlockLocation, state: BlockState) {
        self.global.world_blocks.set_block(location, state);
    }


    fn on_entity_destroy(&mut self, id: u32) {
        self.global.world_entities.remove_entity(id, self.local.bot_id);
    }

    fn on_entity_spawn(&mut self, id: u32, location: Location) {
        self.global.world_entities.put_entity(id, self.local.bot_id, location);
    }

    fn on_disconnect(&mut self, _reason: &str) {
        self.local.disconnected = true;
    }

    fn on_socket_close(&mut self) {}
}
