use radix_engine_interface::math::Decimal;
use radix_engine_interface::math::ParseDecimalError;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::*;
use crate::manifest_type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestDecimal(pub Decimal);

//========
// error
//========

/// Represents an error when parsing ManifestDecimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestDecimalError {
    InvalidDecimal(ParseDecimalError),
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
        let decimal =
            Decimal::try_from(slice).map_err(ParseManifestDecimalError::InvalidDecimal)?;
        Ok(Self(decimal))
    }
}

impl ManifestDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

manifest_type!(ManifestDecimal, ManifestCustomValueKind::Decimal, 32);
