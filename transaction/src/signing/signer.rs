use scrypto::crypto::{EcdsaPublicKey, EcdsaSignature};

use super::EcdsaPrivateKey;

pub trait Signer {
    fn public_key(&self) -> EcdsaPublicKey;

    fn sign(&self, message: &[u8]) -> EcdsaSignature;
}

impl Signer for EcdsaPrivateKey {
    fn public_key(&self) -> EcdsaPublicKey {
        self.public_key()
    }

    fn sign(&self, message: &[u8]) -> EcdsaSignature {
        self.sign(message)
    }
}
