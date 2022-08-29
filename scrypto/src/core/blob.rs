use sbor::rust::convert::TryFrom;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::crypto::*;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Blob(pub Hash);

impl Blob {
    pub fn new(slice: &[u8]) -> Self {
        Self(hash(slice))
    }
}

//========
// error
//========

/// Represents an error when parsing Blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBlobError {
    InvalidHash,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBlobError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Blob {
    type Error = ParseBlobError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(
            Hash::try_from(slice).map_err(|_| Self::Error::InvalidHash)?,
        ))
    }
}

impl Blob {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(Blob, ScryptoType::Blob, Vec::new());

//======
// text
//======

impl FromStr for Blob {
    type Err = ParseBlobError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = Hash::from_str(s).map_err(|_| ParseBlobError::InvalidHash)?;
        Ok(Self(hash))
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}
