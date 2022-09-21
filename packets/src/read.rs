use std::io::{Cursor, Read};

use bytes::Buf;

use crate::types::{BitField, VarUInt};

#[derive(Clone)]
pub struct ByteReader {
    bytes: Cursor<Vec<u8>>,
}

pub struct LenRead<T> {
    pub value: T,
    pub len: usize,
}

impl ByteReader {
    pub fn read<T: ByteReadable>(&mut self) -> T {
        T::read_from_bytes(self)
    }

    pub fn back(&mut self, bytes: u64) {
        let position = self.bytes.position();
        let new_position = position - bytes;
        self.bytes.set_position(new_position);
    }

    pub fn read_with_len<T: ByteReadable>(&mut self) -> LenRead<T> {
        let pos_before = self.bytes.position();
        let value = T::read_from_bytes(self);
        let pos_after = self.bytes.position();

        LenRead {
            value,
            len: (pos_after - pos_before) as usize,
        }
    }

    pub fn read_like<T: ByteReadableLike<Param = P>, P>(&mut self, input: &P) -> T {
        T::read_from_bytes(self, input)
    }

    pub fn empty(&self) -> bool {
        !self.bytes.has_remaining()
    }

    pub fn len(&self) -> usize {
        self.bytes.remaining()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new(vec: Vec<u8>) -> ByteReader {
        let bytes = Cursor::new(vec);
        Self { bytes }
    }
}

impl Read for ByteReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.bytes.read(buf)
    }
}

pub trait ByteReadable {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self;
}

pub trait ByteReadableLike {
    type Param;
    fn read_from_bytes(byte_reader: &mut ByteReader, param: &Self::Param) -> Self;
}

impl ByteReadable for u8 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_u8()
    }
}

impl ByteReadable for i32 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_i32()
    }
}

impl ByteReadable for u32 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_u32()
    }
}

impl ByteReadable for u16 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_u16()
    }
}

impl ByteReadable for f64 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_f64()
    }
}

impl<A: ByteReadable, B: ByteReadable> ByteReadable for (A, B) {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        (byte_reader.read(), byte_reader.read())
    }
}

impl<A: ByteReadable, B: ByteReadable, C: ByteReadable> ByteReadable for (A, B, C) {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        (byte_reader.read(), byte_reader.read(), byte_reader.read())
    }
}

impl<A: ByteReadable, B: ByteReadable, C: ByteReadable, D: ByteReadable> ByteReadable
    for (A, B, C, D)
{
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        (
            byte_reader.read(),
            byte_reader.read(),
            byte_reader.read(),
            byte_reader.read(),
        )
    }
}

impl<A: ByteReadable, B: ByteReadable, C: ByteReadable, D: ByteReadable, E: ByteReadable>
    ByteReadable for (A, B, C, D, E)
{
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        (
            byte_reader.read(),
            byte_reader.read(),
            byte_reader.read(),
            byte_reader.read(),
            byte_reader.read(),
        )
    }
}

impl ByteReadable for f32 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_f32()
    }
}

impl ByteReadable for i8 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_i8()
    }
}

impl ByteReadable for u64 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_u64()
    }
}

impl<const T: usize> ByteReadable for [u8; T] {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let mut bytes = [0; T];
        for item in bytes.iter_mut() {
            *item = byte_reader.read();
        }
        bytes
    }
}

impl ByteReadable for bool {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let byte = byte_reader.bytes.get_u8();
        !matches!(byte, 0)
    }
}

impl ByteReadable for i16 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_i16()
    }
}

impl ByteReadable for u128 {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        byte_reader.bytes.get_u128()
    }
}

impl<T: ByteReadable> ByteReadable for Vec<T> {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let VarUInt(length) = byte_reader.read();
        (0..length).map(|_| byte_reader.read()).collect()
    }
}

impl ByteReadable for String {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let bytes = byte_reader.read();
        String::from_utf8(bytes).unwrap()
    }
}

impl ByteReadable for BitField {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let raw_byte: u8 = byte_reader.read();
        BitField::from(raw_byte)
    }
}
