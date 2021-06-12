use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};
use packets::types::VarInt;
use std::future::Future;

pub trait Readable where Self: Sized {
    fn read<R: AsyncRead + Send>(reader: &R) -> Pin<Box<dyn std::future::Future<Output=Self>>>;
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

pub trait ReadableExt<T> {
    fn read(&self) -> Pin<Box<dyn std::future::Future<Output=T>>>;
}

impl<T: Readable, A: AsyncRead + Send> ReadableExt<T> for A {
    fn read(&self) -> Pin<Box<dyn std::future::Future<Output=T>>> {
        T::read(self)
    }
}

pub trait Writable where Self: Sized {
    fn write<W: AsyncWrite + Unpin>(self, writer: &mut W) -> Pin<Box<dyn std::future::Future<Output=()>>>;
}
