use crate::{
    ecdsa_secp256k1::EcdsaSecp256k1PrivateKey, eddsa_ed25519::EddsaEd25519PrivateKey,
    model::SignatureWithPublicKey,
};
use radix_engine_common::prelude::IsHash;

pub trait Signer {
    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKey;
}

impl Signer for EcdsaSecp256k1PrivateKey {
    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKey {
        self.sign(message_hash).into()
    }
}

impl Signer for EddsaEd25519PrivateKey {
    fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKey {
        (self.public_key(), self.sign(message_hash)).into()
    }
}
