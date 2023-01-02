use sbor::rust::convert::TryFrom;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::*;
use crate::scrypto_type;
use scrypto_abi::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestExpression {
    EntireWorktop,
    EntireAuthZone,
}

impl ManifestExpression {
    pub fn entire_worktop() -> Self {
        Self::EntireWorktop
    }

    pub fn entire_auth_zone() -> Self {
        Self::EntireAuthZone
    }
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
        match self {
            ManifestExpression::EntireWorktop => vec![0],
            ManifestExpression::EntireAuthZone => vec![1],
        }
    }
}

scrypto_type!(
    ManifestExpression,
    ScryptoCustomTypeId::Expression,
    Type::Expression,
    1
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_to_bytes() {
        let s = ManifestExpression::EntireAuthZone;
        let s2 = ManifestExpression::try_from(s.to_vec().as_slice()).unwrap();
        assert_eq!(s2, s);
    }
}
