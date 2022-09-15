use sbor::*;

use super::{
    EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, EddsaEd25519PublicKey, EddsaEd25519Signature,
};

/// Represents any natively supported public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum PublicKey {
    EcdsaSecp256k1(EcdsaSecp256k1PublicKey),
    EddsaEd25519(EddsaEd25519PublicKey),
}

/// Represents any natively supported signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum Signature {
    EcdsaSecp256k1(EcdsaSecp256k1Signature),
    EddsaEd25519(EddsaEd25519Signature),
}

/// Represents any natively supported signature, including public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum SignatureWithPublicKey {
    EcdsaSecp256k1(EcdsaSecp256k1Signature),
    EddsaEd25519(EddsaEd25519PublicKey, EddsaEd25519Signature),
}

impl SignatureWithPublicKey {
    pub fn signature(&self) -> Signature {
        match &self {
            SignatureWithPublicKey::EcdsaSecp256k1(sig) => sig.clone().into(),
            SignatureWithPublicKey::EddsaEd25519(_, sig) => sig.clone().into(),
        }
    }
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

impl From<EcdsaSecp256k1Signature> for Signature {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1(signature)
    }
}

impl From<EddsaEd25519Signature> for Signature {
    fn from(signature: EddsaEd25519Signature) -> Self {
        Self::EddsaEd25519(signature)
    }
}

impl From<EcdsaSecp256k1Signature> for SignatureWithPublicKey {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1(signature)
    }
}

impl From<(EddsaEd25519PublicKey, EddsaEd25519Signature)> for SignatureWithPublicKey {
    fn from(combo: (EddsaEd25519PublicKey, EddsaEd25519Signature)) -> Self {
        Self::EddsaEd25519(combo.0, combo.1)
    }
}
