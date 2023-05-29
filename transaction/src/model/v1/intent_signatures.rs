use super::*;
use crate::internal_prelude::*;

/// Represents any natively supported signature, including public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum SignatureWithPublicKeyV1 {
    EcdsaSecp256k1 {
        signature: EcdsaSecp256k1Signature,
    },
    EddsaEd25519 {
        public_key: EddsaEd25519PublicKey,
        signature: EddsaEd25519Signature,
    },
}

impl SignatureWithPublicKeyV1 {
    pub fn signature(&self) -> SignatureV1 {
        match &self {
            Self::EcdsaSecp256k1 { signature } => signature.clone().into(),
            Self::EddsaEd25519 { signature, .. } => signature.clone().into(),
        }
    }
}

impl From<EcdsaSecp256k1Signature> for SignatureWithPublicKeyV1 {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1 { signature }
    }
}

impl From<(EddsaEd25519PublicKey, EddsaEd25519Signature)> for SignatureWithPublicKeyV1 {
    fn from((public_key, signature): (EddsaEd25519PublicKey, EddsaEd25519Signature)) -> Self {
        Self::EddsaEd25519 {
            public_key,
            signature,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct IntentSignatureV1(pub SignatureWithPublicKeyV1);

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct IntentSignaturesV1 {
    pub signatures: Vec<IntentSignatureV1>,
}

pub type PreparedIntentSignaturesV1 = SummarizedRawFullBody<IntentSignaturesV1>;
