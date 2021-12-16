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

use rand::{random, rngs::OsRng};
use rsa::{BigUint, PaddingScheme, PublicKey, RsaPublicKey};

pub struct Rsa {
    key: RsaPublicKey,
}

impl Rsa {
    pub fn from_der(der: &[u8]) -> Rsa {
        // https://wiki.vg/Protocol_Encryption
        let (n, e) = rsa_der::public_key_from_der(der).unwrap();

        // might be wrong endian
        let (n, e) = (BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e));

        Rsa {
            key: RsaPublicKey::new(n, e).unwrap(),
        }
    }

    pub fn encrypt(&self, elem: &[u8]) -> rsa::errors::Result<Vec<u8>> {
        let mut rng = OsRng;
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        self.key.encrypt(&mut rng, padding, elem)
    }
}

pub fn rand_bits() -> [u8; 16] {
    // TODO: "insecure" use OsRng ... I just couldn't figure out how to get a byte
    // from it

    let mut arr = [0_u8; 16];
    for item in &mut arr {
        *item = random();
    }

    arr
}
