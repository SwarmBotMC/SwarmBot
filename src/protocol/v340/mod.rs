

mod clientbound;
mod serverbound;


// struct PacketData {
//     id: u32,
//     data: Vec<u8>
// }

use crate::error::Res;
use crate::bootstrap::Connection;
use crate::client::instance::{State};
use crate::protocol::{McProtocol, Login};
use crate::protocol::io::reader::PacketReader;
use crate::protocol::io::writer::PacketWriter;
use crate::protocol::types::PacketData;
use crate::protocol::v340::serverbound::HandshakeNextState;
use packets::types::VarInt;

pub struct Protocol {
    reader: PacketReader,
    writer: PacketWriter
}

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: Connection) -> Res<Login<Self>> {

        let host = conn.host;
        let port = conn.port;

        let mut reader = PacketReader::from(conn.read);
        let mut writer = PacketWriter::from(conn.write);

        writer.write(serverbound::Handshake {
            protocol_version: VarInt(340),
            host,
            port,
            next_state: HandshakeNextState::Login
        }).await;


        todo!();


    }

    fn apply_packets(&self, client: &mut State) {
        todo!()
    }

    fn teleport(&mut self) {
        todo!()
    }
}
