use crate::internal_prelude::*;

pub enum PrivateKey {
    Secp256k1(Secp256k1PrivateKey),
    Ed25519(Ed25519PrivateKey),
}

impl PrivateKey {
    pub fn public_key(&self) -> PublicKey {
        match self {
            PrivateKey::Secp256k1(key) => key.public_key().into(),
            PrivateKey::Ed25519(key) => key.public_key().into(),
        }
    }
}

impl From<Secp256k1PrivateKey> for PrivateKey {
    fn from(public_key: Secp256k1PrivateKey) -> Self {
        Self::Secp256k1(public_key)
    }
}

impl From<Ed25519PrivateKey> for PrivateKey {
    fn from(public_key: Ed25519PrivateKey) -> Self {
        Self::Ed25519(public_key)
    }
}
