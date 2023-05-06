use crate::crypto::*;
use crate::types::NodeId;
use crate::*;
use sbor::*;

//===============
// TRAITS + UTILS
//===============

pub trait HasPublicKeyHash {
    type TypedPublicKeyHash: IsPublicKeyHash;

    fn get_hash(&self) -> Self::TypedPublicKeyHash;
}

pub trait IsPublicKeyHash: Copy {
    fn get_hash_bytes(&self) -> &[u8; NodeId::UUID_LENGTH];
    fn into_enum(self) -> PublicKeyHash;
}

pub fn hash_public_key_bytes<T: AsRef<[u8]>>(key_bytes: T) -> [u8; NodeId::UUID_LENGTH] {
    hash(key_bytes).lower_bytes()
}

//===============
// ENUM TYPE
//===============

/// The hash of a given public key.
///
/// In particular, it is the last 29 bytes of Blake2b-256 hash of the public key in the Radix canonical encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum PublicKeyHash {
    EcdsaSecp256k1(EcdsaSecp256k1PublicKeyHash),
    EddsaEd25519(EddsaEd25519PublicKeyHash),
}

impl From<EcdsaSecp256k1PublicKeyHash> for PublicKeyHash {
    fn from(public_key: EcdsaSecp256k1PublicKeyHash) -> Self {
        Self::EcdsaSecp256k1(public_key)
    }
}

impl From<EddsaEd25519PublicKeyHash> for PublicKeyHash {
    fn from(public_key: EddsaEd25519PublicKeyHash) -> Self {
        Self::EddsaEd25519(public_key)
    }
}

impl PublicKeyHash {
    pub fn new_from_public_key(public_key: &PublicKey) -> Self {
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => PublicKeyHash::EcdsaSecp256k1(
                EcdsaSecp256k1PublicKeyHash::new_from_public_key(public_key),
            ),
            PublicKey::EddsaEd25519(public_key) => PublicKeyHash::EddsaEd25519(
                EddsaEd25519PublicKeyHash::new_from_public_key(public_key),
            ),
        }
    }
}

impl IsPublicKeyHash for PublicKeyHash {
    fn get_hash_bytes(&self) -> &[u8; NodeId::UUID_LENGTH] {
        match self {
            PublicKeyHash::EcdsaSecp256k1(value) => value.get_hash_bytes(),
            PublicKeyHash::EddsaEd25519(value) => value.get_hash_bytes(),
        }
    }

    fn into_enum(self) -> PublicKeyHash {
        self
    }
}
