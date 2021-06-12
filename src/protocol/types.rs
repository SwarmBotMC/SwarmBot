use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;

use tokio::io::{AsyncRead};
use crate::protocol::serialization::read::{ByteReader, ByteReadable, ByteReadableLike};
use crate::protocol::transform::Readable;
use crate::protocol::serialization::write::{ByteWritable, ByteWriter};

pub struct PacketData {
    pub id: u32,
    pub reader: ByteReader
}

pub trait Packet {
    const ID: u32;
}

#[derive(Copy, Clone, Debug)]
pub struct VarInt(pub i32);

impl From<i32> for VarInt {
    fn from(input: i32) -> Self {
        VarInt(input)
    }
}

impl From<u32> for VarInt {
    fn from(input: u32) -> Self {
        VarInt(input as i32)
    }
}

impl Display for VarInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[async_trait::async_trait]
impl Readable for VarInt {
    fn read<R: AsyncRead + Send>(_reader: &R) -> Pin<Box<dyn Future<Output=Self>>> {

        async fn run() -> VarInt {
            const PART: u32 = 0x7F;
            let mut size = 0;
            let mut val = 0u32;
            loop {
                let b = 0; //reader.read_u8().await.unwrap() as u32;
                val |= (b & PART) << (size * 7);
                size += 1;
                if size > 5 {
                    panic!("oop");
                }
                if (b & 0x80) == 0 {
                    break;
                }
            }
            VarInt(val as i32)
        }

        Box::pin(run())
    }
}

impl ByteReadable for VarInt {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        const PART: u32 = 0x7F;
        let mut size = 0;
        let mut val = 0u32;
        loop {
            let b: u8 = byte_reader.read();
            let b = b as u32;
            val |= (b & PART) << (size * 7);
            size += 1;
            if size > 5 {
                panic!("oop");
            }
            if (b & 0x80) == 0 {
                break;
            }
        }
        VarInt(val as i32)
    }
}

/// Writes like a Vec but without len
#[derive(Debug)]
pub struct RawVec<T = u8>(pub Vec<T>);

impl<T> RawVec<T> {
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn inner(self) -> Vec<T> {
        self.0
    }
}

impl From<Vec<u8>> for RawVec {
    fn from(data: Vec<u8>) -> Self {
        RawVec(data)
    }
}

impl ByteReadable for RawVec {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let mut inner = Vec::new();
        while !byte_reader.empty() {
            let value: u8 = byte_reader.read();
            inner.push(value);
        }
        RawVec(inner)
    }
}

impl<T: ByteReadable> ByteReadableLike for RawVec<T> {
    type Param = usize;

    fn read_from_bytes(byte_reader: &mut ByteReader, param: &usize) -> Self {
        let len = *param;
        let mut inner: Vec<T> = Vec::with_capacity(len);
        for _ in 0..len {
            inner.push(byte_reader.read());
        }
        RawVec(inner)
    }
}


impl ByteWritable for VarInt {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        const PART: u32 = 0x7F;
        let mut val = self.0 as u32;
        loop {
            if (val & !PART) == 0 {
                writer.write(val as u8);
                return;
            }
            writer.write(((val & PART) | 0x80) as u8);
            val >>= 7;
        }
    }
}

impl ByteWritable for RawVec {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        for value in self.inner() {
            writer.write(value);
        }
    }
}

// impl Writable for VarInt {
//     fn write<W: AsyncWrite + Unpin>(self, writer: &mut W) -> Pin<Box<dyn Future<Output=()>>> {
//
//         let inner = self.0;
//
//         let run = async move {
//             const PART: u32 = 0x7F;
//             let mut val = self.0 as u32;
//             loop {
//                 if (val & !PART) == 0 {
//                     writer.write_u8(val as u8).await.unwrap();
//                     return;
//                 }
//                 writer.write_u8(((val & PART) | 0x80) as u8).await.unwrap();
//                 val >>= 7;
//             }
//
//             return;
//         };
//
//         Box::pin(run)
//     }
//     // fn write<W: AsyncWrite>(self, writer: &W) -> Pin<Box<dyn Future<Output=()>>> {
//     //     async fn run(){
//     //         const PART: u32 = 0x7F;
//     //         let mut val = self.0 as u32;
//     //         loop {
//     //             if (val & !PART) == 0 {
//     //                 buf.write_u8(val as u8)?;
//     //                 return Ok(());
//     //             }
//     //             buf.write_u8(((val & PART) | 0x80) as u8)?;
//     //             val >>= 7;
//     //         }
//     //
//     //     }
//     // }
// }
