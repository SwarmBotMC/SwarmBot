use std::io::{Read, Write};

use aes::Aes128;
use aes::cipher::{AsyncStreamCipher, NewCipher};
use cfb8::Cfb8;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use tokio::io::{BufReader, AsyncRead, ReadBuf};
use tokio::net::tcp::OwnedReadHalf;

use crate::packet::transform::ReadableExt;
use crate::packet::types::{PacketData, VarInt};
use std::task::{Context, Poll};
use std::pin::Pin;

type AesCfb8 = Cfb8<Aes128>;

mod reader;


/// https://github.com/RustCrypto/block-ciphers/issues/28
/// https://docs.rs/cfb-mode/0.7.1/cfb_mode/
/// as per https://wiki.vg/Protocol_Encryption#Symmetric_Encryption the key and iv are the same
struct AES {
    cipher: AesCfb8,
}

impl AES {
    pub fn new(key: &[u8]) -> AES {
        AES {
            cipher: AesCfb8::new_from_slices(key, key).unwrap(),
        }
    }

    pub fn encrypt(&mut self, elem: &mut [u8]) {
        self.cipher.encrypt(elem);
    }

    pub fn decrypt(&mut self, elem: &mut [u8]) {
        self.cipher.decrypt(elem);
    }
}

#[derive(Copy, Clone)]
struct ZLib {
    threshold: u32,
}

const EXPANSION: f64 = 1.5;

/// https://wiki.vg/Protocol#With_compression
impl ZLib {
    fn new(threshold: u32) -> ZLib {
        ZLib {
            threshold
        }
    }
    pub fn decompress(&self, input: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity((input.len() as f64 * EXPANSION) as usize);
        ZlibDecoder::new(input).read_to_end(&mut buf).unwrap();
        buf
    }

    pub fn compress(&self, input: &[u8]) -> tokio::io::Result<Vec<u8>> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(input).unwrap();
        e.finish()
    }
}


