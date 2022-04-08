use scrypto::engine::types::*;

pub trait NonceProvider {
    fn get_nonce<PKS: AsRef<[EcdsaPublicKey]>>(&self, intended_signers: PKS) -> u64;
}
