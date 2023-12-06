use crate::engine::wasm_api::{copy_buffer, crypto_utils};
use radix_engine_common::prelude::{BlsPublicKey, BlsSignature, Hash};

/// Crypto utilities.
#[derive(Debug)]
pub struct CryptoUtils {}

impl CryptoUtils {
    pub fn bls_verify(msg_hash: Hash, public_key: BlsPublicKey, signature: BlsSignature) -> bool {
        unsafe {
            crypto_utils::crypto_utils_bls_verify(
                msg_hash.0.as_ptr(),
                msg_hash.0.len(),
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
