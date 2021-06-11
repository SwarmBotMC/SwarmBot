use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncReadExt, BufReader, ReadBuf};
use tokio::net::tcp::OwnedReadHalf;

use crate::packet::io::{AES, ZLib};
use crate::packet::transform::ReadableExt;
use crate::packet::types::{PacketData, VarInt, RawVec};
use crate::packet::serialization::read::{ByteReader, LenRead};

struct Reader {
    reader: EncryptedReader,
    compression: Option<ZLib>,
}

struct EncryptedReader {
    reader: BufReader<OwnedReadHalf>,
    cipher: Option<AES>,
}

impl AsyncRead for EncryptedReader {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let filled_before = buf.filled().len();
        Pin::new(&mut self.reader).poll_read(cx, buf).map_ok(|_| {
            let filled_after = buf.filled().len();

            let to_encrypt = &mut buf.filled_mut()[filled_before..filled_after];

            if let Some(cipher) = self.cipher.as_mut() {
                cipher.decrypt(to_encrypt)
            }
        })
    }
}

impl Reader {
    fn new(read: OwnedReadHalf) -> Reader {
        let reader = BufReader::new(read);

        let reader = EncryptedReader {
            reader,
            cipher: None
        };

        Reader {
            reader,
            compression: None
        }
    }

    fn encryption(&mut self, key: &[u8]) {
        self.reader.cipher = Some(AES::new(key));
    }

    fn compression(&mut self, threshold: u32) {
        self.compression = Some(ZLib::new(threshold))
    }

    pub async fn read(&mut self) -> PacketData {
        let pkt_len;

        // ignore 0-sized packets
        loop {
            let VarInt(len) = self.reader.read().await;
            if len != 0 {
                pkt_len = len as usize;
                break;
            }
        }

        // the raw bytes with length determined by pkt_len
        let mut data = vec![0_u8; pkt_len];
        self.reader.read_exact(&mut data).await.unwrap();

        let mut reader = ByteReader::new(data);

        let data = match self.compression.as_ref() {
            None => packet_reader(&mut reader, pkt_len),
            Some(zlib) => packet_reader_compressed(&mut reader, zlib, pkt_len)
        };

        let mut reader = ByteReader::new(data);
        let VarInt(id) = reader.read();

        PacketData {
            id: id as u32,
            reader,
        }
    }
}

fn packet_reader_compressed(reader: &mut ByteReader, zlib: &ZLib, len: usize) -> Vec<u8> {
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
