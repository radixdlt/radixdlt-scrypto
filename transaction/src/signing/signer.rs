use crate::{
    ecdsa_secp256k1::EcdsaSecp256k1PrivateKey, eddsa_ed25519::EddsaEd25519PrivateKey,
    model::SignatureWithPublicKey,
};
use radix_engine_interface::crypto::Hash;

pub trait Signer {
    fn sign(&self, message_hash: &Hash) -> SignatureWithPublicKey;
}

impl Signer for EcdsaSecp256k1PrivateKey {
    fn sign(&self, message_hash: &Hash) -> SignatureWithPublicKey {
        self.sign(message_hash).into()
    }
}

impl Signer for EddsaEd25519PrivateKey {
    fn sign(&self, message_hash: &Hash) -> SignatureWithPublicKey {
        (self.public_key(), self.sign(&message_hash)).into()
    }
}
