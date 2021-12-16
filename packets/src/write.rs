// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io::Write;

use bytes::{BufMut, BytesMut};

use crate::types::VarInt;

#[derive(Default)]
pub struct ByteWriter {
    bytes: BytesMut,
}

impl Write for ByteWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for elem in buf {
            self.write(*elem);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl ByteWritable for u8 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_u8(self);
    }
}

impl ByteWritable for bool {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        let val: u8 = if self { 1 } else { 0 };
        writer.write(val);
    }
}

impl ByteWritable for u128 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_u128(self);
    }
}

impl ByteWritable for i16 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_i16(self);
    }
}

impl ByteWritable for f64 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_f64(self);
    }
}

impl ByteWritable for u64 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_u64(self);
    }
}

impl ByteWritable for f32 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_f32(self);
    }
}

impl ByteWritable for u16 {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.bytes.put_u16(self);
    }
}

pub trait ByteWritable {
    fn write_to_bytes(self, writer: &mut ByteWriter);
}

pub trait ByteWritableLike {
    type Param;
    fn write_to_bytes_like(self, writer: &mut ByteWriter, param: &Self::Param);
}

impl ByteWriter {
    pub fn write<T: ByteWritable>(&mut self, value: T) -> &mut Self {
        value.write_to_bytes(self);
        self
    }

    pub fn write_like<T: ByteWritableLike<Param = P>, P>(
        &mut self,
        value: T,
        param: &P,
    ) -> &mut Self {
        value.write_to_bytes_like(self, param);
        self
    }

    pub fn new() -> Self {
        ByteWriter {
            bytes: BytesMut::new(),
        }
    }

    pub fn freeze(self) -> Vec<u8> {
        self.bytes.freeze().to_vec()
    }
}

impl ByteWritable for &[u8] {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        for &x in self {
            writer.write(x);
        }
    }
}

impl ByteWritable for String {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        let bytes = self.as_bytes();
        let byte_len = self.bytes().len();

        writer.write(VarInt::from(byte_len)).write(bytes);
    }
}

impl ByteWritable for Vec<u8> {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        let len: VarInt = self.len().into();
        writer.write(len);
        for x in self {
            writer.write(x);
        }
    }
}
