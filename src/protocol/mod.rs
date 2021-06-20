use crate::bootstrap::Connection;

use crate::error::Res;
use crate::types::Location;


use packets::types::UUID;
use crate::client::processor::InterfaceIn;

pub mod v340;

mod io;
mod transform;
mod encrypt;

pub trait InterfaceOut {
    fn send_chat(&mut self, message: &str);
    fn respawn(&mut self);
    fn teleport(&mut self, location: Location);
}

#[async_trait::async_trait]
pub trait Minecraft: Sized {
    type Queue: EventQueue;
    type Interface: InterfaceOut;
    async fn login(conn: Connection) -> Res<Login<Self::Queue, Self::Interface>>;
}

pub trait EventQueue {
    fn flush(&mut self, processor: &mut impl InterfaceIn);
}

#[derive(Debug)]
pub struct ClientInfo {
    pub username: String,
    pub uuid: UUID,
    pub entity_id: u32,
}


/// login for a given bot. Holds
pub struct Login<E: EventQueue, I: InterfaceOut> {
    pub queue: E,
    pub out: I,
    pub info: ClientInfo
}
