use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

/// Represents any natively supported public key.
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "public_key")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Categorize, Encode, Decode, BasicDescribe)]
pub enum PublicKey {
    Secp256k1(Secp256k1PublicKey),
    Ed25519(Ed25519PublicKey),
}

impl Describe<ScryptoCustomTypeKind> for PublicKey {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::PUBLIC_KEY_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::public_key_type_data()
    }
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
