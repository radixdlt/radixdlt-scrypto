use sbor::*;

use super::{EcdsaPublicKey, EcdsaSignature, Ed25519PublicKey, Ed25519Signature};

/// Represents any natively supported public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum PublicKey {
    Ecdsa(EcdsaPublicKey),
    Ed25519(Ed25519PublicKey),
}

/// Represents any natively supported signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum Signature {
    Ecdsa(EcdsaSignature),
    Ed25519(Ed25519Signature),
}

/// Represents any natively supported signature, including public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum SignatureWithPublicKey {
    Ecdsa(EcdsaSignature),
    Ed25519(Ed25519PublicKey, Ed25519Signature),
}

impl Signature {
    pub fn verify(&self, message: &[u8], public_key: &PublicKey) -> bool {
        todo!()
    }
}

impl SignatureWithPublicKey {
    pub fn public_key(&self) -> PublicKey {
        match &self {
            SignatureWithPublicKey::Ecdsa(_) => todo!("Recover"),
            SignatureWithPublicKey::Ed25519(pk, _) => pk.clone().into(),
        }
    }
    pub fn signature(&self) -> Signature {
        match &self {
            SignatureWithPublicKey::Ecdsa(sig) => sig.clone().into(),
            SignatureWithPublicKey::Ed25519(_, sig) => sig.clone().into(),
        }
    }
    pub fn verify(&self, message: &[u8]) -> bool {
        todo!()
    }
}

impl From<EcdsaPublicKey> for PublicKey {
    fn from(public_key: EcdsaPublicKey) -> Self {
        Self::Ecdsa(public_key)
    }
}

impl From<Ed25519PublicKey> for PublicKey {
    fn from(public_key: Ed25519PublicKey) -> Self {
        Self::Ed25519(public_key)
    }
}

impl From<EcdsaSignature> for Signature {
    fn from(signature: EcdsaSignature) -> Self {
        Self::Ecdsa(signature)
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

impl From<EcdsaSignature> for SignatureWithPublicKey {
    fn from(_signature: EcdsaSignature) -> Self {
        todo!("Recover")
    }
}

impl From<(Ed25519PublicKey, Ed25519Signature)> for SignatureWithPublicKey {
    fn from(combo: (Ed25519PublicKey, Ed25519Signature)) -> Self {
        Self::Ed25519(combo.0, combo.1)
    }
}
