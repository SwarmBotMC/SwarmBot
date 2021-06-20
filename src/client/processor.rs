


use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::protocol::InterfaceOut;
use crate::storage::chunk::ChunkColumn;
use crate::storage::world::ChunkLocation;
use crate::types::{Chat, Location};
use crate::storage::block::BlockApprox;

pub trait InterfaceIn {
    fn on_chat(&mut self, message: Chat);
    fn on_death(&mut self);
    fn on_move(&mut self, location: Location);
    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn);
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
            match player_msg.message {
                "nearby" => {
                    let mut below = self.local.block_location();
                    below.1 -= 1;
                    let below_block = self.global.world_blocks.get_block(below);
                    if let Some(BlockApprox::Realized(below_block)) = below_block {
                        let message = format!("below {:?} is {:?}", self.local.block_location(), below_block.id());
                        self.out.send_chat(&message);
                    }
                }
                _msg => {

                }
            }
        }
    }

    fn on_death(&mut self) {
        self.out.respawn();
        self.out.send_chat("I died... oof... well I guess I should respawn");
    }

    fn on_move(&mut self, location: Location) {
        self.local.location = location;
    }

    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn) {
        self.global.world_blocks.add_column(location, column);
    }

    fn on_disconnect(&mut self, _reason: &str) {
        self.local.disconnected = true;
    }

    fn on_socket_close(&mut self) {

    }
}
