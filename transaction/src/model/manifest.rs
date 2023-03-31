use crate::model::Instruction;
use radix_engine_interface::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
}
