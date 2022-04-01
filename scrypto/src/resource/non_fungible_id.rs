use sbor::*;

use crate::rust::borrow::ToOwned;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a key for a non-fungible resource
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonFungibleId(Vec<u8>);

impl NonFungibleId {
    pub fn new(v: Vec<u8>) -> Self {
        NonFungibleId(v)
    }
}

impl From<u128> for NonFungibleId {
    fn from(u: u128) -> Self {
        NonFungibleId(u.to_be_bytes().to_vec())
    }
}

//========
// error
//========

/// Represents an error when decoding non-fungible id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleIdError {
    InvalidHex(String),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for NonFungibleId {
    type Error = ParseNonFungibleIdError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(slice.to_vec()))
    }
}

impl NonFungibleId {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

custom_type!(NonFungibleId, CustomType::NonFungibleId, Vec::new());

//======
// text
//======

impl FromStr for NonFungibleId {
    type Err = ParseNonFungibleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseNonFungibleIdError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Debug for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
