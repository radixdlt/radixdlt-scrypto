use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct NotarySignatureV1(pub SignatureV1);

pub type PreparedNotarySignatureV1 = SummarizedRawFullBody<NotarySignatureV1>;
