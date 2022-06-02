use scrypto::crypto::EcdsaSignature;

use super::EcdsaPrivateKey;

pub trait Signer {
    fn sign(&self, message: &[u8]) -> (EcdsaPrivateKey, EcdsaSignature);
}

impl Signer for EcdsaPrivateKey {
    fn sign(&self, message: &[u8]) -> (EcdsaPrivateKey, EcdsaSignature) {
        todo!()
    }
}
