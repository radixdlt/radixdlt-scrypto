use crate::engine::wasm_api::{copy_buffer, crypto_utils};
use radix_engine_common::prelude::{Bls12381G1PublicKey, Bls12381G2Signature, Hash};
use sbor::prelude::Vec;

/// Crypto utilities.
#[derive(Debug)]
pub struct CryptoUtils {}

impl CryptoUtils {
    pub fn bls12381_v1_verify(
        message: Vec<u8>,
        public_key: Bls12381G1PublicKey,
        signature: Bls12381G2Signature,
    ) -> bool {
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_verify(
                message.as_ptr(),
                message.len(),
                public_key.0.as_ptr(),
                public_key.0.len(),
                signature.0.as_ptr(),
                signature.0.len(),
            ) != 0
        }
    }

    pub fn keccak_hash(data: Vec<u8>) -> Hash {
        let hash = copy_buffer(unsafe {
            crypto_utils::crypto_utils_keccak_hash(data.as_ptr(), data.len())
        });

        Hash(hash.try_into().unwrap())
    }
}
