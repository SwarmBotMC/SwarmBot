



// struct PacketData {
//     id: u32,
//     data: Vec<u8>
// }

use crate::error::Res;
use crate::bootstrap::Connection;
use crate::client::instance::Client;
use crate::protocol::{McProtocol, ClientProtocol};

pub struct Protocol;

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: &Connection) -> Res<ClientProtocol<Self>> {
        todo!()
    }


    fn apply_packets(&self, client: &mut Client) {
        todo!()
    }

    fn teleport(&mut self) {
        todo!()
    }
}

// #[async_trait::async_trait]
// impl McProtocol for Protocol {
//     type PacketType = PacketData;
//
//     async fn login(read_half: OwnedReadHalf, write_half: OwnedWriteHalf) -> Res<Self> {
//         todo!()
//     }
//
//     async fn read_packet(&self) -> Self::PacketType {
//         todo!()
//     }
//
//     async fn write_packet(&self) -> Self::PacketType {
//         todo!()
//     }
// }
