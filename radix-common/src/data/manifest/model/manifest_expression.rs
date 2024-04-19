#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_rust::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use radix_rust::rust::fmt;
use radix_rust::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use sbor::prelude::vec;

    #[test]
    fn manifest_expression_parse_fail() {
        // wrong length
        let vec_err_1 = vec![1u8, 2];
        // wrong variant id
        let vec_err_2 = vec![10u8];

        let err1 = ManifestExpression::try_from(vec_err_1.as_slice());
        assert!(matches!(
            err1,
            Err(ParseManifestExpressionError::InvalidLength)
        ));
        #[cfg(not(feature = "alloc"))]
        println!("Decoding manifest expression error: {}", err1.unwrap_err());

        let err2 = ManifestExpression::try_from(vec_err_2.as_slice());
        assert!(matches!(
            err2,
            Err(ParseManifestExpressionError::UnknownExpression)
        ));
        #[cfg(not(feature = "alloc"))]
        println!("Decoding manifest expression error: {}", err2.unwrap_err());
    }

    #[test]
    fn manifest_expression_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder.decode_deeper_body_with_value_kind::<ManifestExpression>(
            ManifestExpression::value_kind(),
        );

        assert!(matches!(addr_output, Err(DecodeError::InvalidCustomValue)));
    }
}
