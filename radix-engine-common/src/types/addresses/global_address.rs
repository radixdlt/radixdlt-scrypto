use crate::address::Bech32Decoder;
use crate::address::{AddressDisplayContext, EncodeBech32AddressError, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::Reference;
use crate::data::scrypto::*;
use crate::types::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use sbor::rust::prelude::*;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

/// Address to a global entity
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

#[cfg(feature = "radix_engine_fuzzing")]
// Implementing arbitrary by hand to make sure that EntityType::Global.. marker is present.
// Otherwise 'InvalidCustomValue' error is returned
impl<'a> Arbitrary<'a> for GlobalAddress {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        use core::cmp::min;
        let global_entities: [u8; 14] = [
            EntityType::GlobalPackage as u8,
            EntityType::GlobalFungibleResourceManager as u8,
            EntityType::GlobalNonFungibleResourceManager as u8,
            EntityType::GlobalConsensusManager as u8,
            EntityType::GlobalValidator as u8,
            EntityType::GlobalAccessController as u8,
            EntityType::GlobalAccount as u8,
            EntityType::GlobalIdentity as u8,
            EntityType::GlobalGenericComponent as u8,
            EntityType::GlobalVirtualSecp256k1Account as u8,
            EntityType::GlobalVirtualEd25519Account as u8,
            EntityType::GlobalVirtualSecp256k1Identity as u8,
            EntityType::GlobalVirtualEd25519Identity as u8,
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

impl TryFrom<[u8; NodeId::LENGTH]> for GlobalAddress {
    type Error = ParseGlobalAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        let node_id = NodeId(value);

        if node_id.is_global() {
            Ok(Self(node_id))
        } else {
            Err(ParseGlobalAddressError::InvalidEntityTypeId(value[0]))
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

impl From<GlobalAddress> for crate::data::manifest::model::ManifestAddress {
    fn from(value: GlobalAddress) -> Self {
        Self(value.into())
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
    GLOBAL_ADDRESS_ID,
    global_address_type_data
);

manifest_type!(
    GlobalAddress,
    ManifestCustomValueKind::Address,
    NodeId::LENGTH
);

//======
// text
//======

impl fmt::Debug for GlobalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for GlobalAddress {
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
