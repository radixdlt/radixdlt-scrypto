use sbor::*;
use crate::buffer::{SCRYPTO_TYPE_NFT_KEY};

/// Represents a key for an NFT resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Describe)]
pub struct NftKey(pub u128);


impl TypeId for NftKey {
    fn type_id() -> u8 {
        SCRYPTO_TYPE_NFT_KEY
    }
}