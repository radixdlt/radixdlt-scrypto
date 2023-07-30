#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestBalancedDecimal(pub [u8; 32]);

//========
// error
//========

/// Represents an error when parsing ManifestBalancedDecimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestBalancedDecimalError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestBalancedDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestBalancedDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestBalancedDecimal {
    type Error = ParseManifestBalancedDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(copy_u8_array(slice)))
    }
}

impl ManifestBalancedDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

manifest_type!(
    ManifestBalancedDecimal,
    ManifestCustomValueKind::BalancedDecimal,
    32
);
