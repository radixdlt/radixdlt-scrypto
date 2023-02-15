use crate::model::BasicInstruction;
use crate::*;
use radix_engine_interface::*;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
)]
pub struct TransactionManifest {
    pub instructions: Vec<BasicInstruction>,
    pub blobs: Vec<Vec<u8>>,
}
