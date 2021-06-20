use std::cell::RefCell;
use std::rc::Rc;

use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::protocol::InterfaceOut;
use crate::storage::chunk::ChunkColumn;
use crate::storage::world::ChunkLocation;
use crate::types::{Chat, Location};

pub trait InterfaceIn {
    fn on_chat(&mut self, message: Chat);
    fn on_death(&mut self);
    fn on_move(&mut self, location: Location);
    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn);
    fn on_disconnect(&mut self, reason: &str);
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
            let message = format!("{} sent a message", player_msg.player);
            self.out.send_chat(&message);
        }
    }

    fn on_death(&mut self) {

    }

    fn on_move(&mut self, location: Location) {

    }

    fn on_recv_chunk(&mut self, location: ChunkLocation, column: ChunkColumn) {

    }

    fn on_disconnect(&mut self, reason: &str) {

    }
}
