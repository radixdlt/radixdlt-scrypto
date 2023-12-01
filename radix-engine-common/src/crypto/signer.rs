use crate::internal_prelude::*;
use radix_engine_common::prelude::IsHash;

pub trait Signer {
    fn public_key(&self) -> PublicKey;
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1;
    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1;
}

impl Signer for Secp256k1PrivateKey {
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
        self.sign(message_hash).into()
    }

    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
        self.sign(message_hash).into()
    }

    fn public_key(&self) -> PublicKey {
        self.public_key().into()
    }
}

impl Signer for Ed25519PrivateKey {
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
        self.sign(message_hash).into()
    }

    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
        (self.public_key(), self.sign(message_hash)).into()
    }

    fn public_key(&self) -> PublicKey {
        self.public_key().into()
    }
}

impl Signer for PrivateKey {
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
        match self {
            PrivateKey::Secp256k1(key) => key.sign_without_public_key(message_hash),
            PrivateKey::Ed25519(key) => key.sign_without_public_key(message_hash),
        }
    }

    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
        match self {
            PrivateKey::Secp256k1(key) => key.sign_with_public_key(message_hash),
            PrivateKey::Ed25519(key) => key.sign_with_public_key(message_hash),
        }
    }

    fn public_key(&self) -> PublicKey {
        self.public_key()
    }
}
