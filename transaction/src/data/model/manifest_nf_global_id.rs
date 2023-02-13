use radix_engine_interface::blueprints::resource::NonFungibleLocalId;
use radix_engine_interface::blueprints::resource::ParseNonFungibleLocalIdError;
use radix_engine_interface::blueprints::resource::ResourceAddress;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::*;

use crate::data::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestNonFungibleGlobalId(pub ResourceAddress, pub NonFungibleLocalId);

//========
// error
//========

/// Represents an error when parsing ManifestNonFungibleGlobalId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestNonFungibleGlobalIdError {
    InvalidLength,
    InvalidResourceAddress,
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestNonFungibleGlobalIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestNonFungibleGlobalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl Categorize<ManifestCustomValueKind> for ManifestNonFungibleGlobalId {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestNonFungibleGlobalId
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_slice(&self.0.to_vec())?;
        self.1.encode_body_common(encoder)?;
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestNonFungibleGlobalId
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let resource_address = ResourceAddress::try_from(decoder.read_slice(27)?)
            .map_err(|_| DecodeError::InvalidCustomValue)?;
        let local_id = NonFungibleLocalId::decode_body_common(decoder)?;
        Ok(Self(resource_address, local_id))
    }
}
