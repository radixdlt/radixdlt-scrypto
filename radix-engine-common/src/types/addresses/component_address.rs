use crate::address::Bech32Decoder;
use crate::address::{AddressDisplayContext, EncodeBech32AddressError, NO_NETWORK};
use crate::crypto::{hash, PublicKey};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::Reference;
use crate::data::scrypto::*;
use crate::types::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use sbor::rust::prelude::*;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

/// Address to a global component
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentAddress(NodeId); // private to ensure entity type check

impl ComponentAddress {
    pub const fn new_unchecked(raw: [u8; NodeId::LENGTH]) -> Self {
        Self(NodeId(raw))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn virtual_account_from_public_key<P: Into<PublicKey> + Clone>(
        public_key: &P,
    ) -> ComponentAddress {
        match public_key.clone().into() {
            PublicKey::EcdsaSecp256k1(public_key) => {
                let mut temp = hash(public_key.to_vec()).lower_27_bytes();
                temp[0] = EntityType::GlobalVirtualEcdsaAccount as u8;
                Self(NodeId(temp))
            }
            PublicKey::EddsaEd25519(public_key) => {
                let mut temp = hash(public_key.to_vec()).lower_27_bytes();
                temp[0] = EntityType::GlobalVirtualEddsaAccount as u8;
                Self(NodeId(temp))
            }
        }
    }

    pub fn virtual_identity_from_public_key<P: Into<PublicKey> + Clone>(
        public_key: &P,
    ) -> ComponentAddress {
        match public_key.clone().into() {
            PublicKey::EcdsaSecp256k1(public_key) => {
                let mut temp = hash(public_key.to_vec()).lower_27_bytes();
                temp[0] = EntityType::GlobalVirtualEcdsaIdentity as u8;
                Self(NodeId(temp))
            }
            PublicKey::EddsaEd25519(public_key) => {
                let mut temp = hash(public_key.to_vec()).lower_27_bytes();
                temp[0] = EntityType::GlobalVirtualEddsaIdentity as u8;
                Self(NodeId(temp))
            }
        }
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

impl AsRef<[u8]> for ComponentAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<[u8; NodeId::LENGTH]> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(value: [u8; NodeId::LENGTH]) -> Result<Self, Self::Error> {
        if EntityType::from_repr(value[0])
            .ok_or(ParseComponentAddressError::InvalidEntityTypeId(value[0]))?
            .is_global_component()
        {
            Ok(Self(NodeId(value)))
        } else {
            Err(ParseComponentAddressError::InvalidEntityTypeId(value[0]))
        }
    }
}

impl TryFrom<&[u8]> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => ComponentAddress::try_from(copy_u8_array(slice)),
            _ => Err(ParseComponentAddressError::InvalidLength(slice.len())),
        }
    }
}

impl Into<[u8; NodeId::LENGTH]> for ComponentAddress {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0.into()
    }
}

impl From<ComponentAddress> for super::GlobalAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::new_unchecked(value.into())
    }
}

impl From<ComponentAddress> for Reference {
    fn from(value: ComponentAddress) -> Self {
        Self(value.into())
    }
}

impl From<ComponentAddress> for crate::data::manifest::model::ManifestAddress {
    fn from(value: ComponentAddress) -> Self {
        Self(value.into())
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseComponentAddressError {
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseComponentAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseComponentAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    ComponentAddress,
    ScryptoCustomValueKind::Reference,
    Type::ComponentAddress,
    NodeId::LENGTH,
    COMPONENT_ADDRESS_ID
);

manifest_type!(
    ComponentAddress,
    ManifestCustomValueKind::Address,
    NodeId::LENGTH
);

//======
// text
//======

impl fmt::Debug for ComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ComponentAddress {
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
        write!(f, "ComponentAddress({})", hex::encode(&self.0))
            .map_err(EncodeBech32AddressError::FormatError)
    }
}
