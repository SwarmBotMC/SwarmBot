use crate::bootstrap::Connection;
use crate::client::instance::{Client, ClientInfo, State};
use crate::error::Res;

pub mod v340;

mod types;
mod io;
mod transform;

#[async_trait::async_trait]
pub trait McProtocol where Self: Sized {
    async fn login(conn: Connection) -> Res<Login<Self>>;
    fn apply_packets(&self, client: &mut State);
    fn teleport(&mut self);
}


pub struct Login<T: McProtocol> {
    pub protocol: T,
    pub info: ClientInfo
}
