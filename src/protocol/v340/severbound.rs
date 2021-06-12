use crate::protocol::types::Packet;

pub struct Handshake {
    pub protocol_version: VarInt,
    pub host: String,

    /// hostname or IP
    pub port: u16,

    /// default 25565
    pub next_state: NextState, // 1 for status, 2 for login
}

impl Packet for Handshake {
    const ID: u32 = 0x00;
    const STATE: PacketState = PacketState::Handshake;
}

#[derive(Copy, Clone)]
#[repr(i32)]
pub enum NextState {
    Status = 1,
    Login = 2,
}

impl ByteWritable for NextState {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        let val_i32 = self as i32;
        writer.write(VarInt(val_i32));
    }
}

impl From<NextState> for VarInt {
    fn from(state: NextState) -> Self {
        VarInt(state as i32)
    }
}

impl ByteWritable for Handshake {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer
            .write(self.protocol_version)
            .write(self.host)
            .write(self.port)
            .write(self.next_state);
    }
}
