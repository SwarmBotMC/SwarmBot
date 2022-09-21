use std::{
    fmt::{Display, Formatter},
    pin::Pin,
};

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    read::{ByteReadable, ByteReadableLike, ByteReader},
    write::{ByteWritable, ByteWriter},
};

pub trait Packet {
    const ID: u32;
    const STATE: PacketState;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PacketState {
    Handshake,
    Status,
    Login,
    Play,
}

impl Display for PacketState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            PacketState::Handshake => "handshake",
            PacketState::Status => "status",
            PacketState::Login => "login",
            PacketState::Play => "play",
        };
        f.write_str(res)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VarInt(pub i32);

pub type Angle = u8;
pub type Identifier = String;
pub type Chat = String;

pub struct BitField {
    pub values: [bool; 8],
}

impl From<VarInt> for u32 {
    fn from(elem: VarInt) -> Self {
        elem.0 as u32
    }
}

impl From<VarInt> for i32 {
    fn from(elem: VarInt) -> Self {
        elem.0
    }
}

impl From<u8> for BitField {
    fn from(mut byte: u8) -> BitField {
        let mut values = [false; 8];
        let mut i = 0;
        while byte != 0 {
            let val = byte & 0b10000000 != 0;
            values[i] = val;
            byte <<= 1;
            i += 1;
        }
        BitField { values }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VarUInt(pub usize);

impl From<i32> for VarInt {
    fn from(input: i32) -> Self {
        VarInt(input)
    }
}

/// Writes like a Vec but without len
#[derive(Debug)]
pub struct RawVec<T = u8>(pub Vec<T>);

impl<T> RawVec<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    // TODO: how does ownership inference work here
    #[inline]
    pub fn inner(self) -> Vec<T> {
        self.0
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

impl From<Vec<u8>> for RawVec {
    fn from(data: Vec<u8>) -> Self {
        RawVec(data)
    }
}

impl ByteWritable for RawVec {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        for value in self.inner() {
            writer.write(value);
        }
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

#[derive(Debug, Copy, Clone)]
pub struct UUIDHyphenated(pub u128);

impl ByteReadable for UUIDHyphenated {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let mut str: String = byte_reader.read();
        str = str.replace('-', "");
        UUIDHyphenated(u128::from_str_radix(&str, 16).unwrap())
    }
}

impl From<UUIDHyphenated> for UUID {
    fn from(hyph: UUIDHyphenated) -> Self {
        UUID(hyph.0)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct UUID(pub u128);

impl From<&String> for UUID {
    fn from(s: &String) -> Self {
        let inner = u128::from_str_radix(s, 16).unwrap();
        UUID(inner)
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl ByteWritable for UUID {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.write(self.0);
    }
}

impl ByteReadable for UUID {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let inner: u128 = byte_reader.read();
        UUID(inner)
    }
}

impl From<usize> for VarInt {
    fn from(input: usize) -> Self {
        VarInt(input as i32)
    }
}

impl From<u32> for VarInt {
    fn from(input: u32) -> Self {
        VarInt(input as i32)
    }
}

// pub type NBT = nbt::Blob;

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

impl VarInt {
    pub async fn read_async<R: AsyncRead>(mut reader: Pin<&mut R>) -> VarInt {
        const PART: u32 = 0x7F;
        let mut size = 0;
        let mut val = 0u32;
        loop {
            let b = reader.read_u8().await.unwrap() as u32;
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

impl ByteReadable for VarUInt {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let VarInt(contents) = byte_reader.read();
        VarUInt(contents as usize)
    }
}
