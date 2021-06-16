use packets::types::UUID;

use crate::client::runner::GlobalState;
use crate::protocol::McProtocol;
use crate::storage::world::WorldBlocks;
use crate::types::Location;

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}

pub struct State {
    pub ticks: usize,
    pub info: ClientInfo,
    pub alive: bool,
    pub location: Location
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
}
