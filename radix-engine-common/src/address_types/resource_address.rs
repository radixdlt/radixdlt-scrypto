use crate::address::{AddressDisplayContext, EncodeBech32AddressError, EntityType, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use radix_engine_constants::NODE_ID_LENGTH;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use utils::{copy_u8_array, ContextualDisplay};

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceAddress([u8; NODE_ID_LENGTH]); // private to ensure entity type check

impl ResourceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
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
            NODE_ID_LENGTH => match EntityType::from_repr(slice[0])
                .ok_or(ParseResourceAddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::GlobalNonFungibleResourceManager
                | EntityType::GlobalFungibleResourceManager => Ok(Self(copy_u8_array(&slice[1..]))),
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
    NODE_ID_LENGTH,
    RESOURCE_ADDRESS_ID
);

manifest_type!(
    ResourceAddress,
    ManifestCustomValueKind::Address,
    NODE_ID_LENGTH
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
    type Error = EncodeBech32AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_to_fmt(f, &self.0);
        }

        // This could be made more performant by streaming the hex into the formatter
        write!(f, "ResourceAddress({})", hex::encode(&self.0))
            .map_err(|err| EncodeBech32AddressError::FormatError(err))
    }
}
