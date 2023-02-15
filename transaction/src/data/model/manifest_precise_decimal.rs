use radix_engine_interface::math::ParsePreciseDecimalError;
use radix_engine_interface::math::PreciseDecimal;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::*;
use crate::manifest_type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestPreciseDecimal(pub PreciseDecimal);

//========
// error
//========

/// Represents an error when parsing ManifestPreciseDecimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestPreciseDecimalError {
    InvalidPreciseDecimal(ParsePreciseDecimalError),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestPreciseDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestPreciseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestPreciseDecimal {
    type Error = ParseManifestPreciseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let precise_decimal = PreciseDecimal::try_from(slice)
            .map_err(ParseManifestPreciseDecimalError::InvalidPreciseDecimal)?;
        Ok(Self(precise_decimal))
    }
}

impl ManifestPreciseDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

manifest_type!(
    ManifestPreciseDecimal,
    ManifestCustomValueKind::PreciseDecimal,
    64
);
