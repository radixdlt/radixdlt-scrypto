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

/// Address to a global entity
#[cfg_attr(feature = "fuzzing", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlobalAddress(NodeId); // private to ensure entity type check

impl GlobalAddress {
    pub const fn new_or_panic(raw: [u8; NodeId::LENGTH]) -> Self {
        let node_id = NodeId(raw);
        assert!(node_id.is_global());
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
}

#[cfg(feature = "fuzzing")]
// Implementing arbitrary by hand to make sure that EntityType::Global.. marker is present.
// Otherwise 'InvalidCustomValue' error is returned
impl<'a> Arbitrary<'a> for GlobalAddress {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        use core::cmp::min;
        let global_entities: [u8; 13] = [
            EntityType::GlobalPackage as u8,
            EntityType::GlobalFungibleResourceManager as u8,
            EntityType::GlobalNonFungibleResourceManager as u8,
            EntityType::GlobalConsensusManager as u8,
            EntityType::GlobalValidator as u8,
            EntityType::GlobalAccessController as u8,
            EntityType::GlobalAccount as u8,
            EntityType::GlobalIdentity as u8,
            EntityType::GlobalGenericComponent as u8,
            EntityType::GlobalPreallocatedSecp256k1Account as u8,
            EntityType::GlobalPreallocatedEd25519Account as u8,
            EntityType::GlobalPreallocatedSecp256k1Identity as u8,
            EntityType::GlobalPreallocatedEd25519Identity as u8,
        ];

        let mut node_id = [0u8; NodeId::LENGTH];
        node_id[0] = *u.choose(&global_entities[..]).unwrap();
        // fill NodeId with available random bytes (fill the rest with zeros if data exhausted)
        let len = min(NodeId::LENGTH - 1, u.len());
        let (_left, right) = node_id.split_at_mut(NodeId::LENGTH - len);
        let b = u.bytes(len).unwrap();
        right.copy_from_slice(&b);
        Ok(Self::new_or_panic(node_id))
    }
}

impl AsRef<[u8]> for GlobalAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<NodeId> for GlobalAddress {
    fn as_ref(&self) -> &NodeId {
        &self.0
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for GlobalAddress {
    type Error = ParseGlobalAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        Self::try_from(NodeId(value))
    }
}

impl TryFrom<NodeId> for GlobalAddress {
    type Error = ParseGlobalAddressError;

    fn try_from(node_id: NodeId) -> Result<Self, Self::Error> {
        if node_id.is_global() {
            Ok(Self(node_id))
        } else {
            Err(ParseGlobalAddressError::InvalidEntityTypeId(node_id.0[0]))
        }
    }
}

impl TryFrom<&[u8]> for GlobalAddress {
    type Error = ParseGlobalAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => GlobalAddress::try_from(copy_u8_array(slice)),
            _ => Err(ParseGlobalAddressError::InvalidLength(slice.len())),
        }
    }
}

impl Into<[u8; NodeId::LENGTH]> for GlobalAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl From<GlobalAddress> for Reference {
    fn from(value: GlobalAddress) -> Self {
        Self(value.into())
    }
}

impl From<GlobalAddress> for ManifestAddress {
    fn from(value: GlobalAddress) -> Self {
        Self::Static(value.into())
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseGlobalAddressError {
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseGlobalAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseGlobalAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    GlobalAddress,
    ScryptoCustomValueKind::Reference,
    Type::Address,
    NodeId::LENGTH,
    GLOBAL_ADDRESS_TYPE,
    global_address_type_data
);

impl Categorize<ManifestCustomValueKind> for GlobalAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for GlobalAddress {
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

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for GlobalAddress {
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

//======
// text
//======

impl fmt::Debug for GlobalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for GlobalAddress {
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
        write!(f, "Address({})", hex::encode(&self.0))
            .map_err(AddressBech32EncodeError::FormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    #[test]
    fn global_address_initialization() {
        let node_id = [0; NodeId::LENGTH];
        let addr = unsafe { GlobalAddress::new_unchecked(node_id) };
        assert_eq!(node_id, addr.as_node_id().as_bytes());

        let addr = GlobalAddress::new_or_panic([EntityType::GlobalPackage as u8; NodeId::LENGTH]);
        // validate conversions
        GlobalAddress::try_from_hex(&addr.to_hex()).unwrap();
        Reference::try_from(addr).unwrap();
        let _ = ManifestAddress::try_from(addr).unwrap();

        // pass empty string to fail conversion
        assert!(
            GlobalAddress::try_from_bech32(&AddressBech32Decoder::for_simulator(), "").is_none()
        );

        // pass wrong length array to generate an error
        let v = Vec::from([0u8; NodeId::LENGTH + 1]);
        let addr2 = GlobalAddress::try_from(v.as_slice());
        assert_matches!(addr2, Err(ParseGlobalAddressError::InvalidLength(..)));

        // pass wrong node id (bad entity type) to generate an error
        let v = Vec::from([0u8; NodeId::LENGTH]);
        let addr3 = GlobalAddress::try_from(v.as_slice());
        assert_matches!(addr3, Err(ParseGlobalAddressError::InvalidEntityTypeId(..)));
        #[cfg(not(feature = "alloc"))]
        println!("Decode error: {}", addr3.unwrap_err());
    }

    #[test]
    fn global_address_decode_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder
            .decode_deeper_body_with_value_kind::<GlobalAddress>(GlobalAddress::value_kind());

        assert_matches!(addr_output, Err(DecodeError::InvalidCustomValue));
    }
}
