use scrypto::crypto::SignatureWithPublicKey;

use super::{EcdsaPrivateKey, Ed25519PrivateKey};

pub trait Signer {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey;
}

impl Signer for EcdsaPrivateKey {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey {
        self.sign(message).into()
    }
}

impl Signer for Ed25519PrivateKey {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey {
        (self.public_key(), self.sign(message)).into()
    }
}
