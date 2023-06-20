use crate::crypto::*;
use crate::types::NodeId;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::*;

//===============
// TRAITS + UTILS
//===============

pub trait HasPublicKeyHash {
    type TypedPublicKeyHash: IsPublicKeyHash;

    fn get_hash(&self) -> Self::TypedPublicKeyHash;
}

pub trait IsPublicKeyHash: Copy {
    fn get_hash_bytes(&self) -> &[u8; NodeId::RID_LENGTH];
    fn into_enum(self) -> PublicKeyHash;
}

pub fn hash_public_key_bytes<T: AsRef<[u8]>>(key_bytes: T) -> [u8; NodeId::RID_LENGTH] {
    hash(key_bytes).lower_bytes()
}

//===============
// ENUM TYPE
//===============

/// The hash of a given public key.
///
/// In particular, it is the last 29 bytes of Blake2b-256 hash of the public key in the Radix canonical encoding.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum PublicKeyHash {
    Secp256k1(Secp256k1PublicKeyHash),
    Ed25519(Ed25519PublicKeyHash),
}

impl From<Secp256k1PublicKeyHash> for PublicKeyHash {
    fn from(public_key: Secp256k1PublicKeyHash) -> Self {
        Self::Secp256k1(public_key)
    }
}

impl From<Ed25519PublicKeyHash> for PublicKeyHash {
    fn from(public_key: Ed25519PublicKeyHash) -> Self {
        Self::Ed25519(public_key)
    }
}

impl PublicKeyHash {
    pub fn new_from_public_key(public_key: &PublicKey) -> Self {
        match public_key {
            PublicKey::Secp256k1(public_key) => {
                PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash::new_from_public_key(public_key))
            }
            PublicKey::Ed25519(public_key) => {
                PublicKeyHash::Ed25519(Ed25519PublicKeyHash::new_from_public_key(public_key))
            }
        }
    }
}

impl IsPublicKeyHash for PublicKeyHash {
    fn get_hash_bytes(&self) -> &[u8; NodeId::RID_LENGTH] {
        match self {
            PublicKeyHash::Secp256k1(value) => value.get_hash_bytes(),
            PublicKeyHash::Ed25519(value) => value.get_hash_bytes(),
        }
    }

    fn into_enum(self) -> PublicKeyHash {
        self
    }
}
