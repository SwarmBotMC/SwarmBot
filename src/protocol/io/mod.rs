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

use std::io::{Read, Write};

use aes::{
    cipher::{AsyncStreamCipher, NewCipher},
    Aes128,
};
use cfb8::Cfb8;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

pub mod reader;
pub mod writer;

type AesCfb8 = Cfb8<Aes128>;

/// https://github.com/RustCrypto/block-ciphers/issues/28
/// https://docs.rs/cfb-mode/0.7.1/cfb_mode/
/// as per https://wiki.vg/Protocol_Encryption#Symmetric_Encryption the key and iv are the same
struct Aes {
    cipher: AesCfb8,
}

impl Aes {
    pub fn new(key: &[u8]) -> Aes {
        Aes {
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
        ZLib { threshold }
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
