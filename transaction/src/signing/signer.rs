use crate::{
    ecdsa_secp256k1::EcdsaSecp256k1PrivateKey, eddsa_ed25519::EddsaEd25519PrivateKey,
    model::SignatureWithPublicKey,
};

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
