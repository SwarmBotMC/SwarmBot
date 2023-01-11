use std::io::{Read, Write};

use aes::{
    cipher::{AsyncStreamCipher, NewCipher},
    Aes128,
};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

pub mod reader;
pub mod writer;

type AesCfb8 = cfb8::Cfb8<Aes128>;

/// <https://github.com/RustCrypto/block-ciphers/issues/28>
/// <https://docs.rs/cfb-mode/0.7.1/cfb_mode/>
/// as per <https://wiki.vg/Protocol_Encryption#Symmetric_Encryption> the key and iv are the same
struct Aes {
    cipher: AesCfb8,
}

impl Aes {
    pub fn new(key: &[u8]) -> Self {
        Self {
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

/// <https://wiki.vg/Protocol#With_compression>
#[allow(clippy::unused_self)]
impl ZLib {
    const fn new(threshold: u32) -> Self {
        Self { threshold }
    }
    pub fn decompress(self, input: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity((input.len() as f64 * EXPANSION) as usize);
        ZlibDecoder::new(input).read_to_end(&mut buf).unwrap();
        buf
    }

    pub fn compress(self, input: &[u8]) -> tokio::io::Result<Vec<u8>> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(input).unwrap();
        e.finish()
    }
}
