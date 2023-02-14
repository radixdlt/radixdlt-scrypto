use crate::*;

/// Represents an ECDSA public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub struct EcdsaSecp256k1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl EcdsaSecp256k1PublicKey {
    pub const LENGTH: usize = 33;
}

/// Represents an ED25519 public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub struct EddsaEd25519PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl EddsaEd25519PublicKey {
    pub const LENGTH: usize = 32;
}

/// Represents any natively supported public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "public_key")
)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub enum PublicKey {
    EcdsaSecp256k1(EcdsaSecp256k1PublicKey),
    EddsaEd25519(EddsaEd25519PublicKey),
}

impl From<EcdsaSecp256k1PublicKey> for PublicKey {
    fn from(public_key: EcdsaSecp256k1PublicKey) -> Self {
        Self::EcdsaSecp256k1(public_key)
    }
}

impl From<EddsaEd25519PublicKey> for PublicKey {
    fn from(public_key: EddsaEd25519PublicKey) -> Self {
        Self::EddsaEd25519(public_key)
    }
}
