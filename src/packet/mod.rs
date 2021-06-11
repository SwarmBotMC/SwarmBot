use crate::error::Res;
use crate::connections::Connection;
use crate::client::instance::Client;

pub mod v340;

mod types;
mod io;
mod transform;
mod serialization;

#[async_trait::async_trait]
pub trait McProtocol where Self: Sized {
    type PacketType;
    async fn login(conn: &Connection) -> Res<Self>;
    fn apply_packets(&self, client: &mut Client);
    fn teleport(&mut self);
}
