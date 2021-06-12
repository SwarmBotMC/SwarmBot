use crate::storage::world::WorldBlocks;
use crate::protocol::McProtocol;
use packets::types::UUID;

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}

#[derive(Default)]
pub struct State {

}

pub struct Client<T: McProtocol> {
    pub info: ClientInfo,
    pub state: State,
    pub protocol: T,

}

impl <T: McProtocol> Client<T> {
    pub fn new(info: ClientInfo, world_blocks: &'a WorldBlocks, protocol: T) -> Client<T> {
        Client {
            state: State::default(),
            protocol,
            info,
        }
    }
}
