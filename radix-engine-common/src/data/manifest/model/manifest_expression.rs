use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManifestExpression {
    EntireWorktop,
    EntireAuthZone,
}

//========
// error
//========

/// Represents an error when parsing ManifestExpression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestExpressionError {
    InvalidLength,
    UnknownExpression,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestExpressionError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestExpression {
    type Error = ParseManifestExpressionError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 1 {
            return Err(Self::Error::InvalidLength);
        }
        match slice[0] {
            0 => Ok(Self::EntireWorktop),
            1 => Ok(Self::EntireAuthZone),
            _ => Err(Self::Error::UnknownExpression),
        }
    }
}

impl ManifestExpression {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ManifestExpression::EntireWorktop => {
                bytes.push(0);
            }
            ManifestExpression::EntireAuthZone => {
                bytes.push(1);
            }
        };
        bytes
    }
}

manifest_type!(ManifestExpression, ManifestCustomValueKind::Expression, 1);
