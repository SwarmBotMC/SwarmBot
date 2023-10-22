//! **Block cipher** operates on fixed-length groups of bits is called a
//! **block** Consists of encryption and decryption algorithms

use std::io::{Read, Write};

use aes::{
    cipher::{crypto_common, BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes128,
};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

pub mod reader;
pub mod writer;

type AesEncrypt = cfb8::Encryptor<Aes128>;
type AesDecrypt = cfb8::Decryptor<Aes128>;
type Cfb8Block = crypto_common::Block<AesEncrypt>;

/// <https://github.com/RustCrypto/block-ciphers/issues/28>
/// <https://docs.rs/cfb-mode/0.7.1/cfb_mode/>
/// as per <https://wiki.vg/Protocol_Encryption#Symmetric_Encryption> the key and iv are the same
struct Aes {
    encryptor: AesEncrypt,
    decryptor: AesDecrypt,
}

impl Aes {
    pub fn new(key: &[u8]) -> Self {
        let iv = key;

        let encryptor = AesEncrypt::new_from_slices(key, iv).unwrap();
        let decryptor = AesDecrypt::new_from_slices(key, iv).unwrap();

        Self {
            encryptor,
            decryptor,
        }
    }

    pub fn encrypt(&mut self, elem: &mut [u8]) {
        // SAFETY: Cfb8Block is POD (plain old data)
        let (prefix, blocks, suffix) = unsafe { elem.align_to_mut::<Cfb8Block>() };

        debug_assert!(prefix.is_empty());
        debug_assert!(suffix.is_empty());

        self.encryptor.encrypt_blocks_mut(blocks);
    }

    pub fn decrypt(&mut self, elem: &mut [u8]) {
        // SAFETY: Cfb8Block is POD (plain old data)
        let (prefix, blocks, suffix) = unsafe { elem.align_to_mut::<Cfb8Block>() };

        debug_assert!(prefix.is_empty());
        debug_assert!(suffix.is_empty());

        self.decryptor.decrypt_blocks_mut(blocks);
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
