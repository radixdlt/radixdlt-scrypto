use crate::address::{AddressDisplayContext, EncodeBech32AddressError, EntityType, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::types::NodeId;
use crate::well_known_scrypto_custom_type;
use crate::*;
use radix_engine_constants::NODE_ID_LENGTH;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(NodeId); // private to ensure entity type check

impl Address {
    pub const fn new_unchecked(raw: [u8; NODE_ID_LENGTH]) -> Self {
        Self(NodeId(raw))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl AsRef<NodeId> for Address {
    fn as_ref(&self) -> &NodeId {
        &self.0
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<[u8; NODE_ID_LENGTH]> for Address {
    type Error = ParseAddressError;

    fn try_from(value: [u8; NODE_ID_LENGTH]) -> Result<Self, Self::Error> {
        EntityType::from_repr(value[0]).ok_or(ParseAddressError::InvalidEntityTypeId(value[0]))?;
        Ok(Self(NodeId(value)))
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = ParseAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NODE_ID_LENGTH => Address::try_from(copy_u8_array(slice)),
            _ => Err(ParseAddressError::InvalidLength(slice.len())),
        }
    }
}

impl Into<[u8; NODE_ID_LENGTH]> for Address {
    fn into(self) -> [u8; NODE_ID_LENGTH] {
        self.0.into()
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAddressError {
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    Address,
    ScryptoCustomValueKind::Reference,
    Type::Address,
    NODE_ID_LENGTH,
    ADDRESS_ID
);

manifest_type!(Address, ManifestCustomValueKind::Address, NODE_ID_LENGTH);

//======
// text
//======

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for Address {
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
        write!(f, "Address({})", hex::encode(&self.0))
            .map_err(EncodeBech32AddressError::FormatError)
    }
}
