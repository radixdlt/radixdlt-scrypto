use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::data::manifest::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestDecimal(pub [u8; 32]);

//========
// error
//========

/// Represents an error when parsing ManifestDecimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestDecimalError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestDecimal {
    type Error = ParseManifestDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(copy_u8_array(slice)))
    }
}

impl ManifestDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

manifest_type!(ManifestDecimal, ManifestCustomValueKind::Decimal, 32);
