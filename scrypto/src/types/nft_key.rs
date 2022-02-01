use core::fmt::{Display, Formatter};
use core::fmt;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use crate::buffer::{SCRYPTO_TYPE_NFT_KEY};

/// Represents a key for an NFT resource
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Describe)]
pub struct NftKey(Vec<u8>);

impl NftKey {
    pub fn from_u128(i: u128) -> NftKey {
        NftKey(i.to_le_bytes().to_vec())
    }
}

impl TypeId for NftKey {
    fn type_id() -> u8 {
        SCRYPTO_TYPE_NFT_KEY
    }
}

impl Display for NftKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}