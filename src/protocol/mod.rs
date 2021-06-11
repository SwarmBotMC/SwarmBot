use crate::client::instance::Client;
use crate::bootstrap::Connection;
use crate::error::Res;

pub mod v340;

mod types;
mod io;
mod transform;
mod serialization;

#[async_trait::async_trait]
pub trait McProtocol where Self: Sized {
    async fn login(conn: &Connection) -> Res<ClientProtocol<Self>>;
    fn apply_packets(&self, client: &mut Client);
    fn teleport(&mut self);
}

pub struct ClientProtocol<T: McProtocol> {
    pub protocol: T,
    pub client: Client,
}

impl<T: McProtocol> ClientProtocol<T> {
    pub async fn login(conn: Connection) -> Res<ClientProtocol<T>> {
        T::login(&conn).await
    }
}

