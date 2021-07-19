/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */



use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedSender;

use swarm_bot_packets::types::{Packet, RawVec, VarInt};
use swarm_bot_packets::write::{ByteWritable, ByteWritableLike, ByteWriter};

use crate::error::Res;
use crate::protocol::io::{Aes, ZLib};

pub struct PacketWriter {
    writer: EncryptedWriter,
    compression: Option<ZLib>,
}

struct EncryptedWriter {
    writer: OwnedWriteHalf,
    cipher: Option<Aes>,
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
    fn from(writer: OwnedWriteHalf) -> PacketWriter {
        let writer = EncryptedWriter {
            writer,
            cipher: None,
        };

        PacketWriter {
            writer,
            compression: None,
        }
    }
}

fn data<T: Packet + ByteWritable>(packet: T, compression: &Option<ZLib>) -> Vec<u8> {
    let data = PktData::from(packet);

    let complete_packet = CompletePacket {
        data
    };

    let mut writer = ByteWriter::new();

    complete_packet.write_to_bytes_like(&mut writer, compression);
    writer.freeze()
}

pub struct PacketWriteChannel {
    tx: UnboundedSender<Vec<u8>>,
    compression: Option<ZLib>,
}

impl PacketWriteChannel {
    pub fn write<T: Packet + ByteWritable>(&mut self, packet: T) {
        let data = data(packet, &self.compression);

        self.tx.send(data).unwrap();
    }
}

impl PacketWriter {
    pub fn encryption(&mut self, key: &[u8]) {
        self.writer.cipher = Some(Aes::new(key));
    }

    pub fn compression(&mut self, threshold: u32) {
        self.compression = Some(ZLib::new(threshold))
    }


    pub async fn write<T: Packet + ByteWritable>(&mut self, packet: T) -> Res {
        let mut data = data(packet, &self.compression);
        self.writer.write_all(&mut data).await
    }

    pub fn into_channel(self) -> PacketWriteChannel {
        let compression = self.compression;
        let mut writer = self.writer;
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        tokio::task::spawn_local(async move {
            while let Some(mut elem) = rx.recv().await {
                writer.write_all(&mut elem).await.unwrap();
            }
        });

        PacketWriteChannel {
            tx,
            compression,
        }
    }
}


struct PktData {
    pub id: VarInt,
    pub data: RawVec,
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
