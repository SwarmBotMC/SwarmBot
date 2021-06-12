

mod clientbound;
mod severbound;


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

pub struct Protocol {
    reader: PacketReader,
    writer: PacketWriter
}

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: Connection) -> Res<Login<Self>> {

        let reader = PacketReader::from(conn.read);
        let writer = PacketWriter::from(conn.write);

        todo!()


    }

    fn apply_packets(&self, client: &mut State) {
        todo!()
    }

    fn teleport(&mut self) {
        todo!()
    }
}
