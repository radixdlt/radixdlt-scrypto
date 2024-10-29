use crate::address::AddressBech32Decoder;
use crate::address::{AddressBech32EncodeError, AddressDisplayContext, NO_NETWORK};
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

/// Address to a global resource
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "fuzzing", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceAddress(NodeId); // private to ensure entity type check

impl ResourceAddress {
    pub const fn new_or_panic(raw: [u8; NodeId::LENGTH]) -> Self {
        let node_id = NodeId(raw);
        assert!(node_id.is_global_resource_manager());
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

    pub fn is_fungible(&self) -> bool {
        self.0.is_global_fungible_resource_manager()
    }
}

#[cfg(feature = "fuzzing")]
// Implementing arbitrary by hand to make sure that resource entity type marker is present.
// Otherwise 'InvalidCustomValue' error is returned
impl<'a> Arbitrary<'a> for ResourceAddress {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        use core::cmp::min;
        let resource_entities: [u8; 2] = [
            EntityType::GlobalFungibleResourceManager as u8,
            EntityType::GlobalNonFungibleResourceManager as u8,
        ];

        let mut node_id = [0u8; NodeId::LENGTH];
        node_id[0] = *u.choose(&resource_entities[..]).unwrap();
        // fill NodeId with available random bytes (fill the rest with zeros if data exhausted)
        let len = min(NodeId::LENGTH - 1, u.len());
        let (_left, right) = node_id.split_at_mut(NodeId::LENGTH - len);
        let b = u.bytes(len).unwrap();
        right.copy_from_slice(&b);
        Ok(Self::new_or_panic(node_id))
    }
}

impl AsRef<[u8]> for ResourceAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<NodeId> for ResourceAddress {
    fn as_ref(&self) -> &NodeId {
        &self.0
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        Self::try_from(NodeId(value))
    }
}

impl TryFrom<NodeId> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(node_id: NodeId) -> Result<Self, Self::Error> {
        if node_id.is_global_resource_manager() {
            Ok(Self(node_id))
        } else {
            Err(ParseResourceAddressError::InvalidEntityTypeId(node_id.0[0]))
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

impl TryFrom<GlobalAddress> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(address: GlobalAddress) -> Result<Self, Self::Error> {
        ResourceAddress::try_from(Into::<[u8; NodeId::LENGTH]>::into(address))
    }
}

impl Into<[u8; NodeId::LENGTH]> for ResourceAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl From<ResourceAddress> for super::GlobalAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::new_or_panic(value.into())
    }
}

impl From<ResourceAddress> for Reference {
    fn from(value: ResourceAddress) -> Self {
        Self(value.into())
    }
}

impl From<ResourceAddress> for ManifestAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Static(value.into())
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
    RESOURCE_ADDRESS_TYPE,
    resource_address_type_data
);

impl Categorize<ManifestCustomValueKind> for ResourceAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for ResourceAddress {
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

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for ResourceAddress {
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

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
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
        write!(f, "ResourceAddress({})", hex::encode(&self.0))
            .map_err(|err| AddressBech32EncodeError::FormatError(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    #[test]
    fn resource_address_initialization() {
        let node_id = [0; NodeId::LENGTH];
        let addr = unsafe { ResourceAddress::new_unchecked(node_id) };
        assert_eq!(node_id, addr.as_node_id().as_bytes());

        let addr = ResourceAddress::new_or_panic(
            [EntityType::GlobalNonFungibleResourceManager as u8; NodeId::LENGTH],
        );
        // validate conversions
        ResourceAddress::try_from_hex(&addr.to_hex()).unwrap();
        let _ = ManifestAddress::try_from(addr).unwrap();

        // pass wrong length array to generate an error
        let v = Vec::from([0u8; NodeId::LENGTH + 1]);
        let addr2 = ResourceAddress::try_from(v.as_slice());
        assert_matches!(addr2, Err(ParseResourceAddressError::InvalidLength(..)));

        #[cfg(not(feature = "alloc"))]
        println!("Error: {}", addr2.unwrap_err());
    }

    #[test]
    fn resource_address_decode_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder
            .decode_deeper_body_with_value_kind::<ResourceAddress>(ResourceAddress::value_kind());

        assert_matches!(addr_output, Err(DecodeError::InvalidCustomValue));
    }
}
