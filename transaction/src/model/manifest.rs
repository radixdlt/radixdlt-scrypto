use crate::model::BasicInstruction;
use radix_engine_interface::scrypto;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionManifest {
    pub instructions: Vec<BasicInstruction>,
    pub blobs: Vec<Vec<u8>>,
}
