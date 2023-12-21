use crate::engine::wasm_api::{copy_buffer, crypto_utils};
use radix_engine_common::prelude::{
    scrypto_encode, Bls12381G1PublicKey, Bls12381G2Signature, Hash,
};
use sbor::prelude::Vec;

/// Crypto utilities.
#[derive(Debug)]
pub struct CryptoUtils {}

impl CryptoUtils {
    /// Performs BLS12-381 G2 signature verification.
    /// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
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

    /// Performs BLS12-381 G2 aggregated signature verification of
    /// multiple messages each signed with different key.
    /// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
    pub fn bls12381_v1_aggregate_verify(
        messages: Vec<Vec<u8>>,
        public_keys: Vec<Bls12381G1PublicKey>,
        signature: Bls12381G2Signature,
    ) -> bool {
        let messages: Vec<u8> = scrypto_encode(&messages).unwrap();
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_aggregate_verify(
                messages.as_ptr(),
                messages.len(),
                public_keys.as_ptr() as *const u8,
                public_keys.len() * Bls12381G1PublicKey::LENGTH,
                signature.0.as_ptr(),
                signature.0.len(),
            ) != 0
        }
    }

    /// Aggregate multiple BLS12-381 G2 signatures into single one
    pub fn bls12381_g2_signature_aggregate(
        signatures: Vec<Bls12381G2Signature>,
    ) -> Bls12381G2Signature {
        let agg_signature = copy_buffer(unsafe {
            crypto_utils::crypto_utils_bls12381_g2_signature_aggregate(
                signatures.as_ptr() as *const u8,
                signatures.len() * Bls12381G2Signature::LENGTH,
            )
        });

        Bls12381G2Signature::try_from(agg_signature.as_slice()).unwrap()
    }

    /// Calculates Keccak-256 digest over given vector of bytes
    pub fn keccak256_hash(data: Vec<u8>) -> Hash {
        let hash = copy_buffer(unsafe {
            crypto_utils::crypto_utils_keccak256_hash(data.as_ptr(), data.len())
        });

        Hash(hash.try_into().unwrap())
    }
}
