use rand::{random, rngs::OsRng};
use rsa::{BigUint, Pkcs1v15Encrypt, RsaPublicKey};

pub struct Rsa {
    key: RsaPublicKey,
}

impl Rsa {
    pub fn from_der(der: &[u8]) -> Self {
        // https://wiki.vg/Protocol_Encryption
        let (n, e) = rsa_der::public_key_from_der(der).unwrap();

        // might be wrong endian
        let (n, e) = (BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e));

        Self {
            key: RsaPublicKey::new(n, e).unwrap(),
        }
    }

    pub fn encrypt(&self, elem: &[u8]) -> rsa::errors::Result<Vec<u8>> {
        let mut rng = OsRng;
        self.key.encrypt(&mut rng, Pkcs1v15Encrypt, elem)
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
