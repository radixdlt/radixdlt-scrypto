use super::*;
use crate::internal_prelude::*;

/// Represents any natively supported signature.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "signature")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum SignatureV1 {
    Secp256k1(Secp256k1Signature),
    Ed25519(Ed25519Signature),
}

impl From<Secp256k1Signature> for SignatureV1 {
    fn from(signature: Secp256k1Signature) -> Self {
        Self::Secp256k1(signature)
    }
}

impl From<Ed25519Signature> for SignatureV1 {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct NotarySignatureV1(pub SignatureV1);

#[allow(deprecated)]
pub type PreparedNotarySignatureV1 = SummarizedRawFullValue<NotarySignatureV1>;
