use crate::data::manifest::converter::{from_decimal, to_decimal};
use crate::data::manifest::*;
use crate::math::Decimal;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::prelude::*;
use sbor::*;
use utils::copy_u8_array;

pub const DECIMAL_SIZE: usize = Decimal::BITS / 8;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[cfg_attr(feature = "serde", serde(into = "String"))]
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestDecimal(pub [u8; DECIMAL_SIZE]);

impl TryFrom<String> for ManifestDecimal {
    type Error = crate::internal_prelude::ParseDecimalError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Decimal::from_str(&value).map(|x| from_decimal(&x))
    }
}

impl Into<String> for ManifestDecimal {
    fn into(self) -> String {
        to_decimal(&self).to_string()
    }
}

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
        if slice.len() != DECIMAL_SIZE {
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

manifest_type!(
    ManifestDecimal,
    ManifestCustomValueKind::Decimal,
    DECIMAL_SIZE
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_decimal_parse_fail() {
        let buf = Vec::from_iter(0u8..DECIMAL_SIZE as u8);

        let dec = ManifestDecimal(buf.as_slice().try_into().unwrap());
        let mut dec_vec = dec.to_vec();

        assert!(ManifestDecimal::try_from(dec_vec.as_slice()).is_ok());

        // malform encoded vector
        dec_vec.push(0);
        let dec_out = ManifestDecimal::try_from(dec_vec.as_slice());
        assert!(matches!(
            dec_out,
            Err(ParseManifestDecimalError::InvalidLength)
        ));

        #[cfg(not(feature = "alloc"))]
        println!("Manifest Decimal error: {}", dec_out.unwrap_err());
    }

    #[test]
    fn manifest_decimal_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u8 = 0;
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let dec_output = decoder
            .decode_deeper_body_with_value_kind::<ManifestDecimal>(ManifestDecimal::value_kind());

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert!(matches!(
            dec_output,
            Err(DecodeError::BufferUnderflow { .. })
        ));
    }
}
