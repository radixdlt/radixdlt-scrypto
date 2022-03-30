use scrypto::engine::types::*;

pub trait NonceProvider {
    fn get_nonce(&self, intended_signers: &[EcdsaPublicKey]) -> u64;
}
