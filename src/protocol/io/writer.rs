use std::task::{Poll};

use tokio::io::{ReadBuf, BufWriter, AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use crate::protocol::io::{ZLib, AES};
use crate::protocol::types::{VarInt, RawVec, Packet};
use crate::protocol::serialization::write::{ByteWritable, ByteWritableLike, ByteWriter};
use crate::error::Res;

pub struct PacketWriter {
    writer: EncryptedWriter,
    compression: Option<ZLib>,
}

struct EncryptedWriter {
    writer: BufWriter<OwnedWriteHalf>,
    cipher: Option<AES>,
}

impl EncryptedWriter {
    pub async fn write_all(&mut self, data: &mut [u8]) -> Res<()> {
        if let Some(cipher) = self.cipher.as_mut() {
            cipher.encrypt(data);
        }
        self.writer.write_all(data).await?;
        Ok(())
    }
}

impl From<OwnedWriteHalf> for PacketWriter {
    fn from(write: OwnedWriteHalf) -> PacketWriter {
        let writer = BufWriter::new(write);

        let writer = EncryptedWriter {
            writer,
            cipher: None
        };

        PacketWriter {
            writer,
            compression: None
        }
    }
}

impl PacketWriter {

    pub fn encryption(&mut self, key: &[u8]) {
        self.writer.cipher = Some(AES::new(key));
    }

    pub fn compression(&mut self, threshold: u32) {
        self.compression = Some(ZLib::new(threshold))
    }

    pub async fn write<T: Packet + ByteWritable>(&mut self, packet: T) {
        let data = PktData::from(packet);

        let complete_packet = CompletePacket {
            data
        };

        let mut writer = ByteWriter::new();

        complete_packet.write_to_bytes_like(&mut writer, &self.compression);

        let mut data = writer.freeze();
        self.writer.write_all(&mut data).await.unwrap();

    }
}


struct PktData {
    pub id: VarInt,
    pub data: RawVec
}

impl<T: Packet + ByteWritable> From<T> for PktData {
    fn from(packet: T) -> Self {
        let mut writer = ByteWriter::new();
        writer.write(packet);

        PktData {
            id: VarInt(T::ID as i32),
            data: writer.freeze().into(),
        }
    }
}

struct CompletePacket {
    pub data: PktData,
}

impl ByteWritable for PktData {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.write(self.id)
            .write(self.data);
    }
}

impl ByteWritableLike for CompletePacket {
    type Param = Option<ZLib>;


    fn write_to_bytes_like(self, writer: &mut ByteWriter, zlib: &Self::Param) {
        let mut temp_writer = ByteWriter::new();
        temp_writer.write(self.data);

        let data: RawVec = temp_writer.freeze().into();
        let uncompressed_len = data.len() as i32;


        match zlib {
            None => {
                writer.write(VarInt(uncompressed_len))
                    .write(data);
            }
            Some(zlib) => {
                if uncompressed_len < zlib.threshold as i32 {
                    writer.write(VarInt(uncompressed_len + 1))
                        .write(VarInt(0))
                        .write(data);
                } else {
                    let data: RawVec = zlib.compress(&data.inner()).unwrap().into();
                    let compressed_len: VarInt = (data.len() as i32 + uncompressed_len).into();
                    writer.write(compressed_len)
                        .write(VarInt(uncompressed_len))
                        .write(data);
                }
            }
        }
    }
}
