use crate::crypto::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::*;

/// Represents any natively supported public key.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "public_key")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum PublicKey {
    Secp256k1(Secp256k1PublicKey),
    Ed25519(Ed25519PublicKey),
}

impl From<Secp256k1PublicKey> for PublicKey {
    fn from(public_key: Secp256k1PublicKey) -> Self {
        Self::Secp256k1(public_key)
    }
}

impl From<Ed25519PublicKey> for PublicKey {
    fn from(public_key: Ed25519PublicKey) -> Self {
        Self::Ed25519(public_key)
    }
}

impl HasPublicKeyHash for PublicKey {
    type TypedPublicKeyHash = PublicKeyHash;

    fn get_hash(&self) -> Self::TypedPublicKeyHash {
        PublicKeyHash::new_from_public_key(self)
    }
}
