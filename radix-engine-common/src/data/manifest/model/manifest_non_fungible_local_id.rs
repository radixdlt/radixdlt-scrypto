#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use sbor::rust::prelude::*;
use sbor::*;
use utils::copy_u8_array;

use crate::data::manifest::*;
use crate::*;

// NOTE: redundant code to `NonFungibleLocalId` in favor of minimum dependency

pub const NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestNonFungibleLocalId {
    String(String),
    Integer(u64),
    Bytes(Vec<u8>),
    UUID([u8; 32]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentValidationError {
    TooLong,
    Empty,
    ContainsBadCharacter(char),
    NotUuidV4Variant1,
}

impl ManifestNonFungibleLocalId {
    pub fn string(s: String) -> Result<Self, ContentValidationError> {
        if s.len() == 0 {
            return Err(ContentValidationError::Empty);
        }
        if s.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
            return Err(ContentValidationError::TooLong);
        }
        for char in s.chars() {
            if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
                return Err(ContentValidationError::ContainsBadCharacter(char));
            }
        }
        Ok(Self::String(s))
    }

    pub fn integer(s: u64) -> Result<Self, ContentValidationError> {
        Ok(Self::Integer(s))
    }

    pub fn bytes(s: Vec<u8>) -> Result<Self, ContentValidationError> {
        if s.len() == 0 {
            return Err(ContentValidationError::Empty);
        }
        if s.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
            return Err(ContentValidationError::TooLong);
        }
        Ok(Self::Bytes(s))
    }

    pub fn uuid(s: [u8; 32]) -> Self {
        Self::UUID(s.into())
    }
}

#[cfg(feature = "radix_engine_fuzzing")]
impl<'a> Arbitrary<'a> for ManifestNonFungibleLocalId {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let val = match u.int_in_range(0..=3).unwrap() {
            0 => {
                let charset: Vec<char> =
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZ012345678989_"
                        .chars()
                        .collect();
                let len: u8 = u
                    .int_in_range(1..=NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH as u8)
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
                    .int_in_range(1..=NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH as u8)
                    .unwrap();
                let bytes = (0..len).map(|_| u8::arbitrary(u).unwrap()).collect();
                Self::Bytes(bytes)
            }
            3 => {
                let mut uuid = u128::arbitrary(u).unwrap();
                // make sure this is v4 and variant 1
                uuid &= !0x00000000_0000_f000_c000_000000000000u128;
                uuid |= 0x00000000_0000_4000_8000_000000000000u128;
                Self::UUID(uuid)
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
            Self::UUID(v) => {
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
            3 => Ok(Self::uuid(decoder.read_slice(32)?.try_into().unwrap())),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}
