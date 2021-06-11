use crate::protocol::McProtocol;
use crate::client::instance::Client;
use crate::connections::Connection;
use crate::error::Res;

pub struct ClientProtocol<T: McProtocol> {
    pub protocol: T,
    pub client: Client
}

impl<T: McProtocol> ClientProtocol<T> {
    pub async fn login(conn: Connection) -> Res<ClientProtocol<T>> {
        let protocol = T::login(&conn).await?;

        Ok(ClientProtocol {
            protocol,
            client: Client::default(),
        })
    }
}
