use crate::address::{AddressDisplayContext, AddressError, EntityType, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_constants::NODE_ID_LENGTH;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use utils::{copy_u8_array, ContextualDisplay};

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceAddress([u8; NODE_ID_LENGTH]); // private to ensure entity type check

impl ResourceAddress {
    pub fn to_array_without_entity_id(&self) -> [u8; ADDRESS_HASH_LENGTH] {
        match self {
            Self::Fungible(v) | Self::NonFungible(v) => v.clone(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::resource(self).id());
        match self {
            Self::Fungible(v) | Self::NonFungible(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

impl AsRef<[u8]> for ResourceAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            ADDRESS_LENGTH => match EntityType::try_from(slice[0])
                .map_err(|_| ParseResourceAddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::NonFungibleResource | EntityType::FungibleResource => {
                    Ok(Self(copy_u8_array(&slice[1..])))
                }
                _ => Err(ParseResourceAddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(ParseResourceAddressError::InvalidLength(slice.len())),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResourceAddressError {
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseResourceAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    ResourceAddress,
    ScryptoCustomValueKind::Address,
    Type::ResourceAddress,
    ADDRESS_LENGTH,
    RESOURCE_ADDRESS_ID
);

manifest_type!(
    ResourceAddress,
    ManifestCustomValueKind::Address,
    ADDRESS_LENGTH
);

//========
// text
//========

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_resource_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ResourceAddress::Fungible(_) => {
                write!(f, "FungibleResource[{}]", self.to_hex())
            }
            ResourceAddress::NonFungible(_) => {
                write!(f, "NonFungibleResource[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
