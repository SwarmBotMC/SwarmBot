use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::protocol::InterfaceOut;
use crate::storage::block::{BlockLocation, BlockState};
use crate::storage::chunk::ChunkColumn;
use crate::storage::blocks::ChunkLocation;
use crate::types::{Chat, Location, LocationOrigin};
use crate::client::pathfind::progress_checker::{NoVehicleProgressor, Progressor, Progression};
use crate::client::pathfind::context::{GlobalContext, MoveContext};

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
                        if let [a, b, c] = cmd.args[..] {
                            let x: i64 = a.parse().unwrap();
                            let y: i64 = b.parse().unwrap();
                            let z: i64 = c.parse().unwrap();
                            let dest = BlockLocation(x,y,z);
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
                    "progressions" => {

                        let ctx = GlobalContext {
                            path_config: &self.global.travel_config,
                            world: &self.global.world_blocks,
                        };
                        let prog = NoVehicleProgressor::new(ctx);
                        let loc = MoveContext {
                            location: self.local.physics.location().into(),
                            blocks_can_place: 30
                        };
                        let progressions = prog.progressions(&loc);
                        if let Progression::Movements(neighbors) = progressions {
                            for neighbor in neighbors {
                                self.out.send_chat(&format!("{}", neighbor.value.location));
                            }
                        }
                    }
                    _ => {
                        self.out.send_chat("invalid command");
                    }
                }
            }
            // match player_msg.message {
            //     "nearby" => {
            //         let mut below = self.local.block_location();
            //         below.1 -= 1;
            //         let below_block = self.global.world_blocks.get_block(below);
            //         if let Some(BlockApprox::Realized(below_block)) = below_block {
            //             let message = format!("below {:?} is {:?}", self.local.block_location(), below_block.id());
            //             self.out.send_chat(&message);
            //         }
            //     }
            // }
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
