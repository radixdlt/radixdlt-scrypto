use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct AttachmentsV1 {}

pub type PreparedAttachmentsV1 = SummarizedRawFullBody<AttachmentsV1>;
