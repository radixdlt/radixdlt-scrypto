use super::*;
use crate::internal_prelude::*;

/// Represents any natively supported signature, including public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ManifestSbor, ScryptoSbor)]
pub enum SignatureWithPublicKeyV1 {
    Secp256k1 {
        signature: Secp256k1Signature,
    },
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },
}

impl SignatureWithPublicKeyV1 {
    pub fn signature(&self) -> SignatureV1 {
        match &self {
            Self::Secp256k1 { signature } => signature.clone().into(),
            Self::Ed25519 { signature, .. } => signature.clone().into(),
        }
    }
}

impl From<Secp256k1Signature> for SignatureWithPublicKeyV1 {
    fn from(signature: Secp256k1Signature) -> Self {
        Self::Secp256k1 { signature }
    }
}

impl From<(Ed25519PublicKey, Ed25519Signature)> for SignatureWithPublicKeyV1 {
    fn from((public_key, signature): (Ed25519PublicKey, Ed25519Signature)) -> Self {
        Self::Ed25519 {
            public_key,
            signature,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct IntentSignatureV1(pub SignatureWithPublicKeyV1);

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct IntentSignaturesV1 {
    pub signatures: Vec<IntentSignatureV1>,
}

#[allow(deprecated)]
pub type PreparedIntentSignaturesV1 = SummarizedRawFullValue<IntentSignaturesV1>;
