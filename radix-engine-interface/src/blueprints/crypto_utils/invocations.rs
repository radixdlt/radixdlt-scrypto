use radix_engine_common::prelude::*;

pub const CRYPTO_UTILS_BLUEPRINT: &str = "CryptoUtils";

pub const CRYPTO_UTILS_BLS_VERIFY_IDENT: &str = "bls_verify";
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct CryptoUtilsBlsVerifyInput {
    pub msg_hash: Hash,
    pub pub_key: BlsPublicKey,
    pub signature: BlsSignature,
}
pub type CryptoUtilsBlsVerifyOutput = bool;

pub const CRYPTO_UTILS_KECCAK_HASH_IDENT: &str = "keccak_hash";
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct CryptoUtilsKeccakHashInput {
    pub data: Vec<u8>,
}
pub type CryptoUtilsKeccakHashOutput = Hash;
