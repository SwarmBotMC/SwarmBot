use packets::types::{VarInt, UUIDHyphenated};
use packets::{Readable, Writable, Packet, EnumReadable};


#[derive(Packet, Readable)]
#[packet(0x00, Login)]
pub struct Disconnect {
    pub reason: String,
}

#[derive(Packet, Readable)]
#[packet(0x03, Login)]
pub struct SetCompression {
    pub threshold: VarInt
}
#[derive(Debug, Clone, Packet, Writable, Readable)]
#[packet(0x01, Login)]
pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key_der: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Debug, Packet, Readable)]
#[packet(0x02, Login)]
pub struct LoginSuccess {
    pub uuid: UUIDHyphenated, // 1.16 this is just a raw UUID, 1.12 hyphenated
    pub username: String
}
