use crate::address::EncodeBech32AddressError;
use crate::address::{AddressDisplayContext, EntityType, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::types::NodeId;
use crate::well_known_scrypto_custom_type;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

/// Address to a global package
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackageAddress(NodeId); // private to ensure entity type check

impl PackageAddress {
    pub const fn new_unchecked(raw: [u8; NodeId::LENGTH]) -> Self {
        Self(NodeId(raw))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn as_node_id(&self) -> &NodeId {
        &self.0
    }
}

impl AsRef<[u8]> for PackageAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for PackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        match EntityType::from_repr(value[0])
            .ok_or(ParsePackageAddressError::InvalidEntityTypeId(value[0]))?
        {
            EntityType::GlobalPackage => Ok(Self(NodeId(value))),
            _ => Err(ParsePackageAddressError::InvalidEntityTypeId(value[0])),
        }
    }
}

impl TryFrom<&[u8]> for PackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => PackageAddress::try_from(copy_u8_array(slice)),
            _ => Err(ParsePackageAddressError::InvalidLength(slice.len())),
        }
    }
}

impl Into<[u8; NodeId::LENGTH]> for PackageAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl Into<super::GlobalAddress> for PackageAddress {
    fn into(self) -> super::GlobalAddress {
        super::GlobalAddress::new_unchecked(self.0.into())
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePackageAddressError {
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePackageAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePackageAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    PackageAddress,
    ScryptoCustomValueKind::Reference,
    Type::PackageAddress,
    NodeId::LENGTH,
    PACKAGE_ADDRESS_ID
);

manifest_type!(
    PackageAddress,
    ManifestCustomValueKind::Address,
    NodeId::LENGTH
);

//========
// text
//========

impl fmt::Debug for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for PackageAddress {
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
        write!(f, "PackageAddress({})", hex::encode(&self.0))
            .map_err(|err| EncodeBech32AddressError::FormatError(err))
    }
}
