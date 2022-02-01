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
    pub fn new(v: Vec<u8>) -> Self {
        NftKey(v)
    }
}

impl From<u128> for NftKey {
    fn from(u: u128) -> Self {
        NftKey(u.to_be_bytes().to_vec())
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