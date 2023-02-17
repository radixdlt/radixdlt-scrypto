use crate::model::Instruction;
use transaction_data::*;

#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
}
