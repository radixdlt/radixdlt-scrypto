use radix_common::crypto::*;

pub fn new_secp256k1_private_key(key: u64) -> Secp256k1PrivateKey {
    Secp256k1PrivateKey::from_u64(key).expect("Must succeed!")
}

pub fn new_ed25519_private_key(key: u64) -> Ed25519PrivateKey {
    Ed25519PrivateKey::from_u64(key).expect("Must succeed!")
}
