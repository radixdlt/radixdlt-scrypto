use super::{EcdsaSecp256k1PrivateKey, EddsaEd25519PrivateKey};
use crate::model::SignatureWithPublicKey;
use radix_engine_interface::crypto::hash;

pub trait Signer {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey;
}

impl Signer for EcdsaSecp256k1PrivateKey {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey {
        let message_hash = hash(message).0;
        self.sign(&message_hash).into()
    }
}

impl Signer for EddsaEd25519PrivateKey {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey {
        let message_hash = hash(message).0;
        (self.public_key(), self.sign(&message_hash)).into()
    }
}
