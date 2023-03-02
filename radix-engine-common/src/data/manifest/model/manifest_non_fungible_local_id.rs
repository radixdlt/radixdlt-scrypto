#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
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
    UUID(u128),
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

    pub fn uuid(s: u128) -> Result<Self, ContentValidationError> {
        // 0100 - v4
        // 10 - variant 1
        if (s & 0x00000000_0000_f000_c000_000000000000u128)
            != 0x00000000_0000_4000_8000_000000000000u128
        {
            return Err(ContentValidationError::NotUuidV4Variant1);
        }
        Ok(Self::UUID(s))
    }
}

//========
// error
//========

/// Represents an error when parsing ManifestNonFungibleLocalId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestNonFungibleLocalIdError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestNonFungibleLocalIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestNonFungibleLocalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
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
                encoder.write_slice(&v.to_be_bytes())?;
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
            3 => Self::uuid(u128::from_be_bytes(copy_u8_array(decoder.read_slice(16)?)))
                .map_err(|_| DecodeError::InvalidCustomValue),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}
