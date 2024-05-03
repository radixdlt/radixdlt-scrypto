use crate::address::{AddressBech32Decoder, AddressBech32EncodeError};
use crate::address::{AddressDisplayContext, NO_NETWORK};
use crate::data::manifest::model::ManifestAddress;
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::Reference;
use crate::data::scrypto::*;
use crate::types::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
#[cfg(feature = "fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use radix_rust::{copy_u8_array, ContextualDisplay};
use sbor::rust::prelude::*;
use sbor::*;

/// Address to a global package
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackageAddress(NodeId); // private to ensure entity type check

impl PackageAddress {
    pub const fn new_or_panic(raw: [u8; NodeId::LENGTH]) -> Self {
        let node_id = NodeId(raw);
        assert!(node_id.is_global_package());
        Self(node_id)
    }

    pub unsafe fn new_unchecked(raw: [u8; NodeId::LENGTH]) -> Self {
        Self(NodeId(raw))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    pub fn as_node_id(&self) -> &NodeId {
        &self.0
    }

    pub const fn into_node_id(self) -> NodeId {
        self.0
    }

    pub fn try_from_hex(s: &str) -> Option<Self> {
        hex::decode(s)
            .ok()
            .and_then(|x| Self::try_from(x.as_ref()).ok())
    }

    pub fn try_from_bech32(decoder: &AddressBech32Decoder, s: &str) -> Option<Self> {
        if let Ok((_, full_data)) = decoder.validate_and_decode(s) {
            Self::try_from(full_data.as_ref()).ok()
        } else {
            None
        }
    }

    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    #[cfg(feature = "resource_tracker")]
    pub fn is_native_package(&self) -> bool {
        self.0 == crate::constants::PACKAGE_PACKAGE.0
            || self.0 == crate::constants::RESOURCE_PACKAGE.0
            || self.0 == crate::constants::ACCOUNT_PACKAGE.0
            || self.0 == crate::constants::IDENTITY_PACKAGE.0
            || self.0 == crate::constants::CONSENSUS_MANAGER_PACKAGE.0
            || self.0 == crate::constants::ACCESS_CONTROLLER_PACKAGE.0
            || self.0 == crate::constants::POOL_PACKAGE.0
            || self.0 == crate::constants::TRANSACTION_PROCESSOR_PACKAGE.0
            || self.0 == crate::constants::METADATA_MODULE_PACKAGE.0
            || self.0 == crate::constants::ROYALTY_MODULE_PACKAGE.0
            || self.0 == crate::constants::ROLE_ASSIGNMENT_MODULE_PACKAGE.0
            || self.0 == crate::constants::TRANSACTION_TRACKER_PACKAGE.0
            || self.0 == crate::constants::LOCKER_PACKAGE.0
    }
}

#[cfg(feature = "fuzzing")]
// Implementing arbitrary by hand to make sure that EntityType::GlobalPackage marker is present.
// Otherwise 'InvalidCustomValue' error is returned
impl<'a> Arbitrary<'a> for PackageAddress {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        use core::cmp::min;

        let mut node_id = [0u8; NodeId::LENGTH];
        node_id[0] = EntityType::GlobalPackage as u8;
        // fill NodeId with available random bytes (fill the rest with zeros if data exhausted)
        let len = min(NodeId::LENGTH - 1, u.len());
        let (_left, right) = node_id.split_at_mut(NodeId::LENGTH - len);
        let b = u.bytes(len).unwrap();
        right.copy_from_slice(&b);
        Ok(Self::new_or_panic(node_id))
    }
}

impl AsRef<[u8]> for PackageAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<NodeId> for PackageAddress {
    fn as_ref(&self) -> &NodeId {
        &self.0
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for PackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        Self::try_from(NodeId(value))
    }
}

impl TryFrom<NodeId> for PackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(node_id: NodeId) -> Result<Self, Self::Error> {
        if node_id.is_global_package() {
            Ok(Self(node_id))
        } else {
            Err(ParsePackageAddressError::InvalidEntityTypeId(node_id.0[0]))
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

impl TryFrom<GlobalAddress> for PackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(address: GlobalAddress) -> Result<Self, Self::Error> {
        PackageAddress::try_from(Into::<[u8; NodeId::LENGTH]>::into(address))
    }
}

impl Into<[u8; NodeId::LENGTH]> for PackageAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl From<PackageAddress> for super::GlobalAddress {
    fn from(value: PackageAddress) -> Self {
        Self::new_or_panic(value.into())
    }
}

impl From<PackageAddress> for Reference {
    fn from(value: PackageAddress) -> Self {
        Self(value.into())
    }
}

impl From<PackageAddress> for ManifestAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
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
    PACKAGE_ADDRESS_TYPE,
    package_address_type_data
);

impl Categorize<ManifestCustomValueKind> for PackageAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for PackageAddress {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_discriminator(0)?;
        encoder.write_slice(self.as_ref())?;
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for PackageAddress {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            0 => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

//========
// text
//========

impl fmt::Debug for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for PackageAddress {
    type Error = AddressBech32EncodeError;

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
            .map_err(|err| AddressBech32EncodeError::FormatError(err))
    }
}
