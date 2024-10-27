#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_rust::copy_u8_array;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::math::PreciseDecimal;
use crate::*;

pub const PRECISE_DECIMAL_SIZE: usize = PreciseDecimal::BITS / 8;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestPreciseDecimal(pub [u8; PRECISE_DECIMAL_SIZE]);

//========
// error
//========

/// Represents an error when parsing ManifestPreciseDecimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestPreciseDecimalError {
    InvalidLength,
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
        if slice.len() != PRECISE_DECIMAL_SIZE {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(copy_u8_array(slice)))
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
    PRECISE_DECIMAL_SIZE,
);
scrypto_describe_for_manifest_type!(
    ManifestPreciseDecimal,
    PRECISE_DECIMAL_TYPE,
    precise_decimal_type_data,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    #[test]
    fn manifest_precise_decimal_parse_fail() {
        let buf = Vec::from_iter(0u8..PRECISE_DECIMAL_SIZE as u8);

        let dec = ManifestPreciseDecimal(buf.as_slice().try_into().unwrap());
        let mut dec_vec = dec.to_vec();

        assert!(ManifestPreciseDecimal::try_from(dec_vec.as_slice()).is_ok());

        // malform encoded vector
        dec_vec.push(0);
        let dec_out = ManifestPreciseDecimal::try_from(dec_vec.as_slice());
        assert_matches!(
            dec_out,
            Err(ParseManifestPreciseDecimalError::InvalidLength)
        );

        #[cfg(not(feature = "alloc"))]
        println!("Manifest Precise Decimal error: {}", dec_out.unwrap_err());
    }

    #[test]
    fn manifest_precise_decimal_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u8 = 0;
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let dec_output = decoder.decode_deeper_body_with_value_kind::<ManifestPreciseDecimal>(
            ManifestPreciseDecimal::value_kind(),
        );

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert_matches!(dec_output, Err(DecodeError::BufferUnderflow { .. }));
    }
}
