use crate::engine::wasm_api::{copy_buffer, crypto_utils};
use radix_common::prelude::{
    scrypto_decode, scrypto_encode, Bls12381G1PublicKey, Bls12381G2Signature, Hash,
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
        let public_key: Vec<u8> = scrypto_encode(&public_key).unwrap();
        let signature: Vec<u8> = scrypto_encode(&signature).unwrap();
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_verify(
                message.as_ptr(),
                message.len(),
                public_key.as_ptr(),
                public_key.len(),
                signature.as_ptr(),
                signature.len(),
            ) != 0
        }
    }

    /// Performs BLS12-381 G2 aggregated signature verification of
    /// multiple messages each signed with different key.
    /// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
    pub fn bls12381_v1_aggregate_verify(
        pub_keys_and_msgs: Vec<(Bls12381G1PublicKey, Vec<u8>)>,
        signature: Bls12381G2Signature,
    ) -> bool {
        let pub_keys_and_msgs: Vec<u8> = scrypto_encode(&pub_keys_and_msgs).unwrap();
        let signature: Vec<u8> = scrypto_encode(&signature).unwrap();
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_aggregate_verify(
                pub_keys_and_msgs.as_ptr(),
                pub_keys_and_msgs.len(),
                signature.as_ptr(),
                signature.len(),
            ) != 0
        }
    }

    /// Performs BLS12-381 G2 aggregated signature verification
    /// one message signed with multiple keys.
    /// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
    pub fn bls12381_v1_fast_aggregate_verify(
        message: Vec<u8>,
        public_keys: Vec<Bls12381G1PublicKey>,
        signature: Bls12381G2Signature,
    ) -> bool {
        let public_keys: Vec<u8> = scrypto_encode(&public_keys).unwrap();
        let signature: Vec<u8> = scrypto_encode(&signature).unwrap();
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_fast_aggregate_verify(
                message.as_ptr(),
                message.len(),
                public_keys.as_ptr(),
                public_keys.len(),
                signature.as_ptr(),
                signature.len(),
            ) != 0
        }
    }

    /// Aggregate multiple BLS12-381 G2 signatures into single one
    pub fn bls12381_g2_signature_aggregate(
        signatures: Vec<Bls12381G2Signature>,
    ) -> Bls12381G2Signature {
        let signatures: Vec<u8> = scrypto_encode(&signatures).unwrap();
        let agg_signature = copy_buffer(unsafe {
            crypto_utils::crypto_utils_bls12381_g2_signature_aggregate(
                signatures.as_ptr(),
                signatures.len(),
            )
        });

        scrypto_decode::<Bls12381G2Signature>(&agg_signature).unwrap()
    }

    /// Calculates Keccak-256 digest over given vector of bytes
    pub fn keccak256_hash(data: Vec<u8>) -> Hash {
        let hash = copy_buffer(unsafe {
            crypto_utils::crypto_utils_keccak256_hash(data.as_ptr(), data.len())
        });

        Hash(hash.try_into().unwrap())
    }
}
