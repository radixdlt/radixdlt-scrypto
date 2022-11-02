use crate::model::Instruction;
use sbor::*;
use scrypto::data::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
}
