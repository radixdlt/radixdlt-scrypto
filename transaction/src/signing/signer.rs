use scrypto::crypto::{EcdsaPublicKey, EcdsaSignature};

use super::EcdsaPrivateKey;

pub trait Signer {
    fn sign(&self, message: &[u8]) -> (EcdsaPublicKey, EcdsaSignature);
}

impl Signer for EcdsaPrivateKey {
    fn sign(&self, message: &[u8]) -> (EcdsaPublicKey, EcdsaSignature) {
        let signature = self.sign(message);
        (self.public_key(), signature)
    }
}
