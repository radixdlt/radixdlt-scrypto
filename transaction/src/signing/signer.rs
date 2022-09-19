use scrypto::crypto::SignatureWithPublicKey;

use super::{EcdsaSecp256k1PrivateKey, EddsaEd25519PrivateKey};

pub trait Signer {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey;
}

impl Signer for EcdsaSecp256k1PrivateKey {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey {
        self.sign(message).into()
    }
}

impl Signer for EddsaEd25519PrivateKey {
    fn sign(&self, message: &[u8]) -> SignatureWithPublicKey {
        (self.public_key(), self.sign(message)).into()
    }
}
