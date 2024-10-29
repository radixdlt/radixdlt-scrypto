#[cfg(feature = "fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use radix_rust::copy_u8_array;
use sbor::rust::prelude::*;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

// NOTE: redundant code to `NonFungibleLocalId` in favor of minimum dependency

pub const MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestNonFungibleLocalId {
    String(String),
    Integer(u64),
    Bytes(Vec<u8>),
    RUID([u8; 32]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestNonFungibleLocalIdValidationError {
    TooLong,
    Empty,
    ContainsBadCharacter(char),
}

impl ManifestNonFungibleLocalId {
    pub fn string(s: String) -> Result<Self, ManifestNonFungibleLocalIdValidationError> {
        if s.len() == 0 {
            return Err(ManifestNonFungibleLocalIdValidationError::Empty);
        }
        if s.len() > MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
            return Err(ManifestNonFungibleLocalIdValidationError::TooLong);
        }
        for char in s.chars() {
            if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
                return Err(ManifestNonFungibleLocalIdValidationError::ContainsBadCharacter(char));
            }
        }
        Ok(Self::String(s))
    }

    pub fn integer(s: u64) -> Result<Self, ManifestNonFungibleLocalIdValidationError> {
        Ok(Self::Integer(s))
    }

    pub fn bytes(s: Vec<u8>) -> Result<Self, ManifestNonFungibleLocalIdValidationError> {
        if s.len() == 0 {
            return Err(ManifestNonFungibleLocalIdValidationError::Empty);
        }
        if s.len() > MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
            return Err(ManifestNonFungibleLocalIdValidationError::TooLong);
        }
        Ok(Self::Bytes(s))
    }

    pub fn ruid(s: [u8; 32]) -> Self {
        Self::RUID(s.into())
    }
}

#[cfg(feature = "fuzzing")]
impl<'a> Arbitrary<'a> for ManifestNonFungibleLocalId {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let val = match u.int_in_range(0..=3).unwrap() {
            0 => {
                let charset: Vec<char> =
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZ012345678989_"
                        .chars()
                        .collect();
                let len: u8 = u
                    .int_in_range(1..=MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH as u8)
                    .unwrap();
                let s = (0..len).map(|_| *u.choose(&charset[..]).unwrap()).collect();
                Self::String(s)
            }
            1 => {
                let int = u64::arbitrary(u).unwrap();
                Self::Integer(int)
            }
            2 => {
                let len: u8 = u
                    .int_in_range(1..=MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH as u8)
                    .unwrap();
                let bytes = (0..len).map(|_| u8::arbitrary(u).unwrap()).collect();
                Self::Bytes(bytes)
            }
            3 => {
                let ruid = <[u8; 32]>::arbitrary(u).unwrap();
                Self::RUID(ruid)
            }
            _ => unreachable!(),
        };

        Ok(val)
    }
}

//========
// binary
//========

impl Categorize<ManifestCustomValueKind> for ManifestNonFungibleLocalId {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestNonFungibleLocalId
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::String(v) => {
                encoder.write_discriminator(0)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_bytes())?;
            }
            Self::Integer(v) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(&v.to_be_bytes())?; // TODO: variable length encoding?
            }
            Self::Bytes(v) => {
                encoder.write_discriminator(2)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_slice())?;
            }
            Self::RUID(v) => {
                encoder.write_discriminator(3)?;
                encoder.write_slice(v.as_slice())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestNonFungibleLocalId
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            0 => {
                let size = decoder.read_size()?;
                Self::string(
                    String::from_utf8(decoder.read_slice(size)?.to_vec())
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                )
                .map_err(|_| DecodeError::InvalidCustomValue)
            }
            1 => Self::integer(u64::from_be_bytes(copy_u8_array(decoder.read_slice(8)?)))
                .map_err(|_| DecodeError::InvalidCustomValue),
            2 => {
                let size = decoder.read_size()?;
                Self::bytes(decoder.read_slice(size)?.to_vec())
                    .map_err(|_| DecodeError::InvalidCustomValue)
            }
            3 => Ok(Self::ruid(decoder.read_slice(32)?.try_into().unwrap())),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    #[test]
    fn manifest_non_fungible_local_id_from_string_fail() {
        let too_long_string = ['a' as u8; MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH + 1];

        assert_matches!(
            ManifestNonFungibleLocalId::string(String::new()).unwrap_err(),
            ManifestNonFungibleLocalIdValidationError::Empty
        );
        assert_matches!(
            ManifestNonFungibleLocalId::string(
                String::from_utf8(too_long_string.to_vec()).unwrap()
            )
            .unwrap_err(),
            ManifestNonFungibleLocalIdValidationError::TooLong
        );
        assert_matches!(
            ManifestNonFungibleLocalId::string(String::from("$@!")).unwrap_err(),
            ManifestNonFungibleLocalIdValidationError::ContainsBadCharacter(..)
        );
    }

    #[test]
    fn manifest_non_fungible_local_id_from_bytes_fail() {
        let too_long_buffer = [0u8; MANIFEST_NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH + 1];

        assert_matches!(
            ManifestNonFungibleLocalId::bytes(Vec::<u8>::new()).unwrap_err(),
            ManifestNonFungibleLocalIdValidationError::Empty
        );
        assert_matches!(
            ManifestNonFungibleLocalId::bytes(too_long_buffer.to_vec()).unwrap_err(),
            ManifestNonFungibleLocalIdValidationError::TooLong
        );
    }

    #[test]
    fn manifest_non_fungible_local_id_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let id_output = decoder.decode_deeper_body_with_value_kind::<ManifestNonFungibleLocalId>(
            ManifestNonFungibleLocalId::value_kind(),
        );

        assert_matches!(id_output, Err(DecodeError::InvalidCustomValue));
    }
}
