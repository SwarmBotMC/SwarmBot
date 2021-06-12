use std::fmt::{Display, Formatter};
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use crate::protocol::transform::Readable;
use packets::read::{ByteReader, ByteReadable, ByteReadableLike};
use packets::write::{ByteWritable, ByteWriter};
use std::io::Read;
use std::io;
use crate::error::Res;
use std::future::Future;

pub struct PacketData {
    pub id: u32,
    pub reader: ByteReader
}
