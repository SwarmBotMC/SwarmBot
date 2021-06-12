use crate::storage::world::WorldBlocks;
use crate::protocol::McProtocol;

pub struct ClientInfo {
    username: String,
    uuid: u128,
    entity_id: u32,
}

#[derive(Default)]
pub struct State {

}

pub struct Client<'a, T: McProtocol> {
    pub info: ClientInfo,
    pub state: State,
    pub protocol: T,
    pub world_blocks: &'a WorldBlocks,
}

impl <T: McProtocol> Client<'a, T> {
    pub fn new(info: ClientInfo, world_blocks: &'a WorldBlocks, protocol: T) -> Client<'a, T> {
        Client {
            state: State::default(),
            protocol,
            info,
            world_blocks,
        }
    }
}
