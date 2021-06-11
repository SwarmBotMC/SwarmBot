use crate::connections::Connection;
use crate::error::Res;
use crate::packet::McProtocol;

#[derive(Default, Debug)]
pub struct State {
    tester: String,
}

#[derive(Default)]
pub struct Client {
}

impl Client {
    pub fn run(&self, _scope: &rayon::Scope){
        
    }
}

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


