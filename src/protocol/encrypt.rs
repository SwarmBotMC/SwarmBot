/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use rand::random;
use rand::rngs::OsRng;
use rsa::{BigUint, PaddingScheme, PublicKey, RSAPublicKey};

pub struct RSA {
    key: RSAPublicKey,
}


impl RSA {
    pub fn from_der(der: &[u8]) -> RSA {

        // https://wiki.vg/Protocol_Encryption
        let (n, e) = rsa_der::public_key_from_der(der).unwrap();

        // might be wrong endian
        let (n, e) = (BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e));

        RSA {
            key: RSAPublicKey::new(n, e).unwrap()
        }
    }

    pub fn encrypt(&self, elem: &[u8]) -> rsa::errors::Result<Vec<u8>> {
        let mut rng = OsRng;
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        self.key.encrypt(&mut rng, padding, elem)
    }
}

pub fn rand_bits() -> [u8; 16] {
    // TODO: "insecure" use OsRng ... I just couldn't figure out how to get a byte from it

    let mut arr = [0_u8; 16];
    for i in 0..16 {
        arr[i] = random();
    }

    arr
}
