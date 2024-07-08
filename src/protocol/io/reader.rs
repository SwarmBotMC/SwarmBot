use std::{
    pin::Pin,
    task::{Context, Poll},
};

use swarm_bot_packets::{
    read::{ByteReadable, ByteReader, LenRead},
    types::{Packet, RawVec, VarInt},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, BufReader, ReadBuf},
    net::tcp::OwnedReadHalf,
};

use crate::{
    protocol::io::{Aes, ZLib},
    types::PacketData,
};

pub struct PacketReader {
    reader: EncryptedReader,
    compression: Option<ZLib>,
}

struct EncryptedReader {
    reader: BufReader<OwnedReadHalf>,
    cipher: Option<Aes>,
}

impl AsyncRead for EncryptedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let filled_before = buf.filled().len();
        Pin::new(&mut self.reader).poll_read(cx, buf).map_ok(|()| {
            let filled_after = buf.filled().len();

            let to_encrypt = &mut buf.filled_mut()[filled_before..filled_after];

            if let Some(cipher) = self.cipher.as_mut() {
                cipher.decrypt(to_encrypt);
            }
        })
    }
}

impl From<OwnedReadHalf> for PacketReader {
    fn from(read: OwnedReadHalf) -> Self {
        let reader = BufReader::new(read);

        let reader = EncryptedReader {
            reader,
            cipher: None,
        };

        Self {
            reader,
            compression: None,
        }
    }
}

impl PacketReader {
    pub fn encryption(&mut self, key: &[u8]) {
        self.reader.cipher = Some(Aes::new(key));
    }

    pub fn compression(&mut self, threshold: u32) {
        self.compression = Some(ZLib::new(threshold));
    }

    pub async fn read(&mut self) -> anyhow::Result<PacketData> {
        let pkt_len;

        // ignore 0-sized packets
        loop {
            let len = VarInt::read_async(Pin::new(&mut self.reader)).await;
            let len = len.0;
            if len != 0 {
                pkt_len = len as usize;
                break;
            }
        }

        // the raw bytes with length determined by pkt_len
        let mut data = vec![0_u8; pkt_len];
        self.reader.read_exact(&mut data).await.unwrap();

        let mut reader = ByteReader::new(data);

        let data = match self.compression {
            None => packet_reader(&mut reader, pkt_len),
            Some(zlib) => packet_reader_compressed(&mut reader, zlib, pkt_len),
        };

        let mut reader = ByteReader::new(data);
        let VarInt(id) = reader.read();

        Ok(PacketData {
            id: id as u32,
            reader,
        })
    }

    pub async fn read_exact_packet<T>(&mut self) -> anyhow::Result<T>
    where
        T: Packet + ByteReadable,
    {
        let PacketData { id, mut reader } = self.read().await?;

        if id != T::ID {
            let state = T::STATE;
            let expected = T::ID;
            anyhow::bail!(
                "received the wrong packet! state: {state}, expected: {expected}, actual: {id}"
            )
        }

        let packet = T::read_from_bytes(&mut reader);
        Ok(packet)
    }
}

fn packet_reader_compressed(reader: &mut ByteReader, zlib: ZLib, len: usize) -> Vec<u8> {
    let data: LenRead<VarInt> = reader.read_with_len();

    let len_left = len - data.len;

    let RawVec(inner) = reader.read_like(&len_left);

    if data.value.0 == 0 {
        inner
    } else {
        zlib.decompress(&inner)
    }
}

fn packet_reader(byte_reader: &mut ByteReader, len: usize) -> Vec<u8> {
    let RawVec(inner) = byte_reader.read_like(&len);
    inner
}
