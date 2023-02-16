use crate::model::BasicInstruction;
use transaction_data::*;

#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub struct TransactionManifest {
    pub instructions: Vec<BasicInstruction>,
    pub blobs: Vec<Vec<u8>>,
}
