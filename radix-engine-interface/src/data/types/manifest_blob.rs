use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::crypto::Hash;
use crate::data::*;
use crate::scrypto_type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestBlob(pub Hash);

//========
// error
//========

/// Represents an error when parsing ManifestBlob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestBlobError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestBlobError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestBlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestBlob {
    type Error = ParseManifestBlobError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(Hash(copy_u8_array(slice))))
    }
}

impl ManifestBlob {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(ManifestBlob, ScryptoCustomValueKind::Blob, 32);
