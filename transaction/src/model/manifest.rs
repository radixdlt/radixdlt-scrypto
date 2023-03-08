//use crate::data::{manifest_decode, manifest_encode};
use crate::model::Instruction;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::*;
use sbor::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
}

impl TransactionManifest {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        manifest_decode(slice)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(self)
    }
}
