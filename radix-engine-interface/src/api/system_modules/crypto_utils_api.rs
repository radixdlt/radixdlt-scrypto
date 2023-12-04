use crate::crypto::*;
use crate::internal_prelude::Vec;

pub trait ClientCryptoUtilsApi<E> {
    fn bls_verify(
        &mut self,
        msg_hash: Hash,
        public_key: BlsPublicKey,
        signature: BlsSignature,
    ) -> Result<u32, E>;

    fn keccak_hash(&mut self, data: Vec<u8>) -> Result<Hash, E>;
}
