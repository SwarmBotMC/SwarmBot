use crate::storage::world::WorldBlocks;
use crate::protocol::McProtocol;
use packets::types::UUID;

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}

pub struct State {
    pub info: ClientInfo,
    pub alive: bool
}

pub struct Client<T: McProtocol> {
    pub state: State,
    pub protocol: T,

}
