use crate::address::Bech32Decoder;
use crate::address::{AddressDisplayContext, EncodeBech32AddressError, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::Reference;
use crate::data::scrypto::*;
use crate::types::NodeId;
use crate::types::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use sbor::rust::prelude::*;
use utils::{copy_u8_array, ContextualDisplay};

/// Address to a global resource
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceAddress(NodeId); // private to ensure entity type check

impl ResourceAddress {
    pub const fn new_unchecked(raw: [u8; NodeId::LENGTH]) -> Self {
        Self(NodeId(raw))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn as_node_id(&self) -> &NodeId {
        &self.0
    }

    pub fn try_from_hex(s: &str) -> Option<Self> {
        hex::decode(s)
            .ok()
            .and_then(|x| Self::try_from(x.as_ref()).ok())
    }

    pub fn try_from_bech32(decoder: &Bech32Decoder, s: &str) -> Option<Self> {
        if let Ok((_, full_data)) = decoder.validate_and_decode(s) {
            Self::try_from(full_data.as_ref()).ok()
        } else {
            None
        }
    }

    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
}

impl AsRef<[u8]> for ResourceAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        if EntityType::from_repr(value[0])
            .ok_or(ParseResourceAddressError::InvalidEntityTypeId(value[0]))?
            .is_global_resource()
        {
            Ok(Self(NodeId(value)))
        } else {
            Err(ParseResourceAddressError::InvalidEntityTypeId(value[0]))
        }
    }
}

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => ResourceAddress::try_from(copy_u8_array(slice)),
            _ => Err(ParseResourceAddressError::InvalidLength(slice.len())),
        }
    }
}

impl Into<[u8; NodeId::LENGTH]> for ResourceAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl From<ResourceAddress> for super::GlobalAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::new_unchecked(value.into())
    }
}

impl From<ResourceAddress> for Reference {
    fn from(value: ResourceAddress) -> Self {
        Self(value.into())
    }
}

impl From<ResourceAddress> for crate::data::manifest::model::ManifestAddress {
    fn from(value: ResourceAddress) -> Self {
        Self(value.into())
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
    ScryptoCustomValueKind::Reference,
    Type::ResourceAddress,
    NodeId::LENGTH,
    RESOURCE_ADDRESS_ID
);

manifest_type!(
    ResourceAddress,
    ManifestCustomValueKind::Address,
    NodeId::LENGTH
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
            return encoder.encode_to_fmt(f, self.0.as_ref());
        }

        // This could be made more performant by streaming the hex into the formatter
        write!(f, "ResourceAddress({})", hex::encode(&self.0))
            .map_err(|err| EncodeBech32AddressError::FormatError(err))
    }
}
