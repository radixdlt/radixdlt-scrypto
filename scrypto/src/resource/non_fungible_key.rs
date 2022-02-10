use sbor::{describe::Type, *};

use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a key for a non-fungible resource
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonFungibleKey(Vec<u8>);

impl NonFungibleKey {
    pub fn new(v: Vec<u8>) -> Self {
        NonFungibleKey(v)
    }
}

impl From<u128> for NonFungibleKey {
    fn from(u: u128) -> Self {
        NonFungibleKey(u.to_be_bytes().to_vec())
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseNonFungibleKeyError {
    InvalidHex(hex::FromHexError),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for NonFungibleKey {
    type Error = ParseNonFungibleKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(slice.to_vec()))
    }
}

impl NonFungibleKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

custom_type!(NonFungibleKey, CustomType::NonFungibleKey, Vec::new());

//======
// text
//======

impl FromStr for NonFungibleKey {
    type Err = ParseNonFungibleKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseNonFungibleKeyError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl ToString for NonFungibleKey {
    fn to_string(&self) -> String {
        hex::encode(&self.0)
    }
}

impl fmt::Debug for NonFungibleKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
