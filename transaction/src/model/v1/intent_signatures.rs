use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct IntentSignatureV1(pub SignatureWithPublicKey);

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct IntentSignaturesV1 {
    pub signatures: Vec<IntentSignatureV1>,
}

pub type PreparedIntentSignaturesV1 = SummarizedRawFullBody<IntentSignaturesV1>;
