use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};

pub trait Readable where Self: Sized {
    fn read<R: AsyncRead + Send>(reader: &R) -> Pin<Box<dyn std::future::Future<Output=Self>>>;
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
