



// struct PacketData {
//     id: u32,
//     data: Vec<u8>
// }

use crate::error::Res;
use crate::bootstrap::Connection;
use crate::client::instance::{Client, State};
use crate::protocol::{McProtocol, Login};

pub struct Protocol;

#[async_trait::async_trait]
impl McProtocol for Protocol {
    async fn login(conn: Connection) -> Res<Login<Self>> {
        todo!()
    }

    fn apply_packets(&self, client: &mut State) {
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
