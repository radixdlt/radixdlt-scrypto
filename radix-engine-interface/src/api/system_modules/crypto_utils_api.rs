use crate::crypto::*;
use crate::internal_prelude::Vec;

pub trait ClientCryptoUtilsApi<E> {
    fn bls12381_v1_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Bls12381G1PublicKey,
        signature: Bls12381G2Signature,
    ) -> Result<u32, E>;

    fn keccak256_hash(&mut self, data: Vec<u8>) -> Result<Hash, E>;
}
