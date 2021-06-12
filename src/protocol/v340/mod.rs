

mod clientbound;
mod serverbound;


// struct PacketData {
//     id: u32,
//     data: Vec<u8>
// }

use crate::error::Res;
use crate::bootstrap::{Connection, User};
use crate::client::instance::{State};
use crate::protocol::{McProtocol, Login};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::PacketWriter;
use crate::protocol::types::PacketData;
use crate::protocol::v340::serverbound::HandshakeNextState;
use packets::types::VarInt;
use crate::bootstrap::mojang::AuthResponse;

pub struct Protocol {
    reader: PacketReader,
    writer: PacketWriter
}

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: Connection) -> Res<Login<Self>> {

        let Connection{user, online, mojang, port, read, write, host} = conn;

        let User {email, password, online} = user;


        let mut reader = PacketReader::from(read);
        let mut writer = PacketWriter::from(write);

        let AuthResponse {access_token, name, uuid} = if online {
            mojang.authenticate(&email, &password).await?
        } else {
            let mut response = AuthResponse::default();
            response.name = email;
            response
        };

        // START: handshake
        writer.write(serverbound::Handshake {
            protocol_version: VarInt(340),
            host,
            port,
            next_state: HandshakeNextState::Login
        }).await;

        // START: login
        // writer.write(serverbound::LoginStart {
        //     username: &name
        // });

        if online {
            let clientbound::EncryptionRequest { public_key_der, verify_token, server_id } = reader.read_exact_packet().await?;

            println!("got encryption request {}", server_id);
        }


        todo!()


    }

    fn apply_packets(&self, client: &mut State) {
        todo!()
    }

    fn teleport(&mut self) {
        todo!()
    }
}
