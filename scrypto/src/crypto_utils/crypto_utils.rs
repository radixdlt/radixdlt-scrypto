use crate::engine::wasm_api::{copy_buffer, crypto_utils};
use radix_common::{
    crypto::{
        Ed25519PublicKey, Ed25519Signature, Secp256k1PublicKey, Secp256k1Signature,
        Secp256k1UncompressedPublicKey,
    },
    prelude::{scrypto_decode, scrypto_encode, Bls12381G1PublicKey, Bls12381G2Signature, Hash},
};
use sbor::prelude::Vec;

/// Crypto utilities.
#[derive(Debug)]
pub struct CryptoUtils {}

impl CryptoUtils {
    /// Performs BLS12-381 G2 signature verification.
    /// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
    pub fn bls12381_v1_verify(
        message: impl AsRef<[u8]>,
        public_key: impl AsRef<Bls12381G1PublicKey>,
        signature: impl AsRef<Bls12381G2Signature>,
    ) -> bool {
        let public_key: Vec<u8> = scrypto_encode(public_key.as_ref()).unwrap();
        let signature: Vec<u8> = scrypto_encode(signature.as_ref()).unwrap();
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_verify(
                message.as_ref().as_ptr(),
                message.as_ref().len(),
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
        signature: impl AsRef<Bls12381G2Signature>,
    ) -> bool {
        let pub_keys_and_msgs: Vec<u8> = scrypto_encode(&pub_keys_and_msgs).unwrap();
        let signature: Vec<u8> = scrypto_encode(signature.as_ref()).unwrap();
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
        message: impl AsRef<[u8]>,
        public_keys: impl AsRef<Vec<Bls12381G1PublicKey>>,
        signature: impl AsRef<Bls12381G2Signature>,
    ) -> bool {
        let public_keys: Vec<u8> = scrypto_encode(public_keys.as_ref()).unwrap();
        let signature: Vec<u8> = scrypto_encode(signature.as_ref()).unwrap();
        unsafe {
            crypto_utils::crypto_utils_bls12381_v1_fast_aggregate_verify(
                message.as_ref().as_ptr(),
                message.as_ref().len(),
                public_keys.as_ptr(),
                public_keys.len(),
                signature.as_ptr(),
                signature.len(),
            ) != 0
        }
    }

    /// Aggregate multiple BLS12-381 G2 signatures into single one
    pub fn bls12381_g2_signature_aggregate(
        signatures: impl AsRef<Vec<Bls12381G2Signature>>,
    ) -> Bls12381G2Signature {
        let signatures: Vec<u8> = scrypto_encode(signatures.as_ref()).unwrap();
        let agg_signature = copy_buffer(unsafe {
            crypto_utils::crypto_utils_bls12381_g2_signature_aggregate(
                signatures.as_ptr(),
                signatures.len(),
            )
        });

        scrypto_decode::<Bls12381G2Signature>(&agg_signature).unwrap()
    }

    /// Calculates Keccak-256 digest over given vector of bytes
    pub fn keccak256_hash(data: impl AsRef<[u8]>) -> Hash {
        let hash = copy_buffer(unsafe {
            crypto_utils::crypto_utils_keccak256_hash(data.as_ref().as_ptr(), data.as_ref().len())
        });

        Hash(hash.try_into().unwrap())
    }

    /// Calculates Blake2b-256 digest over given vector of bytes
    pub fn blake2b_256_hash(data: impl AsRef<[u8]>) -> Hash {
        let hash = copy_buffer(unsafe {
            crypto_utils::crypto_utils_blake2b_256_hash(data.as_ref().as_ptr(), data.as_ref().len())
        });

        Hash(hash.try_into().unwrap())
    }

    /// Performs Ed25519 signature verification.
    pub fn ed25519_verify(
        message: impl AsRef<[u8]>,
        public_key: impl AsRef<Ed25519PublicKey>,
        signature: impl AsRef<Ed25519Signature>,
    ) -> bool {
        unsafe {
            crypto_utils::crypto_utils_ed25519_verify(
                message.as_ref().as_ptr(),
                message.as_ref().len(),
                public_key.as_ref().0.as_ptr(),
                public_key.as_ref().0.len(),
                signature.as_ref().0.as_ptr(),
                signature.as_ref().0.len(),
            ) != 0
        }
    }

    /// Performs ECDSA Secp256k1 signature verification.
    pub fn secp256k1_ecdsa_verify(
        message_hash: impl AsRef<Hash>,
        public_key: impl AsRef<Secp256k1PublicKey>,
        signature: impl AsRef<Secp256k1Signature>,
    ) -> bool {
        unsafe {
            crypto_utils::crypto_utils_secp256k1_ecdsa_verify(
                message_hash.as_ref().0.as_ptr(),
                message_hash.as_ref().0.len(),
                public_key.as_ref().0.as_ptr(),
                public_key.as_ref().0.len(),
                signature.as_ref().0.as_ptr(),
                signature.as_ref().0.len(),
            ) != 0
        }
    }

    /// Performs ECDSA Secp256k1 signature verification and public key recovery.
    pub fn secp256k1_ecdsa_verify_and_key_recover(
        message_hash: impl AsRef<Hash>,
        signature: impl AsRef<Secp256k1Signature>,
    ) -> Secp256k1PublicKey {
        let key = copy_buffer(unsafe {
            crypto_utils::crypto_utils_secp256k1_ecdsa_verify_and_key_recover(
                message_hash.as_ref().0.as_ptr(),
                message_hash.as_ref().0.len(),
                signature.as_ref().0.as_ptr(),
                signature.as_ref().0.len(),
            )
        });
        Secp256k1PublicKey(key.try_into().unwrap())
    }

    pub fn secp256k1_ecdsa_verify_and_key_recover_uncompressed(
        message_hash: impl AsRef<Hash>,
        signature: impl AsRef<Secp256k1Signature>,
    ) -> Secp256k1UncompressedPublicKey {
        let key = copy_buffer(unsafe {
            crypto_utils::crypto_utils_secp256k1_ecdsa_verify_and_key_recover_uncompressed(
                message_hash.as_ref().0.as_ptr(),
                message_hash.as_ref().0.len(),
                signature.as_ref().0.as_ptr(),
                signature.as_ref().0.len(),
            )
        });
        Secp256k1UncompressedPublicKey(key.try_into().unwrap())
    }
}
