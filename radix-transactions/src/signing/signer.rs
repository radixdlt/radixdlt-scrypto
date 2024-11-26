use crate::internal_prelude::*;
use radix_common::prelude::IsHash;

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

pub trait Signer {
    fn public_key(&self) -> PublicKey;
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1;
    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1;
}

impl<'a, S: Signer> Signer for &'a S {
    fn public_key(&self) -> PublicKey {
        (*self).public_key()
    }

    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
        (*self).sign_without_public_key(message_hash)
    }

    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
        (*self).sign_with_public_key(message_hash)
    }
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
