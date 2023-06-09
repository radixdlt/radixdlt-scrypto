use crate::internal_prelude::*;
use radix_engine_common::prelude::IsHash;

pub enum PrivateKey {
    Secp256k1(EcdsaSecp256k1PrivateKey),
    Ed25519(EddsaEd25519PrivateKey),
}

impl PrivateKey {
    pub fn public_key(&self) -> PublicKey {
        match self {
            PrivateKey::Secp256k1(key) => key.public_key().into(),
            PrivateKey::Ed25519(key) => key.public_key().into(),
        }
    }
}

impl From<EcdsaSecp256k1PrivateKey> for PrivateKey {
    fn from(public_key: EcdsaSecp256k1PrivateKey) -> Self {
        Self::Secp256k1(public_key)
    }
}

impl From<EddsaEd25519PrivateKey> for PrivateKey {
    fn from(public_key: EddsaEd25519PrivateKey) -> Self {
        Self::Ed25519(public_key)
    }
}

pub trait Signer {
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1;
    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1;
}

impl Signer for EcdsaSecp256k1PrivateKey {
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
        self.sign(message_hash).into()
    }

    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
        self.sign(message_hash).into()
    }
}

impl Signer for EddsaEd25519PrivateKey {
    fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
        self.sign(message_hash).into()
    }

    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
        (self.public_key(), self.sign(message_hash)).into()
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
}
