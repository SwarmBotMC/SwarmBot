use packets::{EnumReadable, Packet};
use packets::types::{Packet, PacketState, VarInt};
use packets::write::{ByteWritable, ByteWriter};

#[derive(Packet)]
#[packet(0x00, Handshake)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub host: String,

    /// hostname or IP
    pub port: u16,

    /// default 25565
    pub next_state: HandshakeNextState, // 1 for status, 2 for login
}

#[derive(Copy, Clone, EnumReadable)]
#[repr(i32)]
pub enum HandshakeNextState {
    Invalid,
    Status,
    Login,
}
