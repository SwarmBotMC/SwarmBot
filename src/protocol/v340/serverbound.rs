use packets::{EnumWritable, Packet, Writable};
use packets::types::{Packet, PacketState, VarInt};
use packets::write::{ByteWritable, ByteWriter};

#[derive(Packet, Writable)]
#[packet(0x00, Handshake)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub host: String,

    /// hostname or IP
    pub port: u16,

    /// default 25565
    pub next_state: HandshakeNextState, // 1 for status, 2 for login
}

#[derive(Copy, Clone, EnumWritable)]
#[repr(i32)]
pub enum HandshakeNextState {

    #[deprecated]
    Invalid,

    Status,
    Login,
}

#[derive(Debug, Packet)]
#[packet(0x00, Login)]
pub struct LoginStart<'a> {
    /// player's username
    pub username: &'a String,
}

#[derive(Packet, Writable)]
#[packet(0x01, Login)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}
