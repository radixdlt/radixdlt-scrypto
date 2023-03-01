use crate::model::Instruction;
use radix_engine_interface::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
}
