use crate::address::{AddressDisplayContext, EncodeBech32AddressError, EntityType, NO_NETWORK};
use crate::crypto::{hash, PublicKey};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use radix_engine_constants::NODE_ID_LENGTH;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentAddress([u8; NODE_ID_LENGTH]); // private to ensure entity type check

impl ComponentAddress {
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
                Self(temp)
            }
            PublicKey::EddsaEd25519(public_key) => {
                let mut temp = hash(public_key.to_vec()).lower_27_bytes();
                temp[0] = EntityType::GlobalVirtualEddsaAccount as u8;
                Self(temp)
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
                Self(temp)
            }
            PublicKey::EddsaEd25519(public_key) => {
                let mut temp = hash(public_key.to_vec()).lower_27_bytes();
                temp[0] = EntityType::GlobalVirtualEddsaIdentity as u8;
                Self(temp)
            }
        }
    }
}

impl AsRef<[u8]> for ComponentAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NODE_ID_LENGTH => match EntityType::from_repr(slice[0])
                .ok_or(ParseComponentAddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::GlobalPackage | // TODO: overlap with PackageAddress?
                EntityType::GlobalFungibleResourceManager | // TODO: overlap with ResourceAddress?
                EntityType::GlobalNonFungibleResourceManager |  // TODO: overlap with ResourceAddress?
                EntityType::GlobalEpochManager |
                EntityType::GlobalValidator |
                EntityType::GlobalClock |
                EntityType::GlobalAccessController |
                EntityType::GlobalAccount |
                EntityType::GlobalIdentity |
                EntityType::GlobalComponent |
                EntityType::GlobalVirtualEcdsaAccount |
                EntityType::GlobalVirtualEddsaAccount |
                EntityType::GlobalVirtualEcdsaIdentity |
                EntityType::GlobalVirtualEddsaIdentity => Ok(Self(copy_u8_array(&slice[1..]))),
                EntityType::InternalVault |
                EntityType::InternalAccessController |
                EntityType::InternalAccount |
                EntityType::InternalComponent |
                EntityType::InternalKeyValueStore => Err(ParseComponentAddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(ParseComponentAddressError::InvalidLength(slice.len())),
        }
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
    ScryptoCustomValueKind::Address,
    Type::ComponentAddress,
    NODE_ID_LENGTH,
    COMPONENT_ADDRESS_ID
);

manifest_type!(
    ComponentAddress,
    ManifestCustomValueKind::Address,
    NODE_ID_LENGTH
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
            return encoder.encode_to_fmt(f, &self.0);
        }

        // This could be made more performant by streaming the hex into the formatter
        write!(f, "ComponentAddress({})", hex::encode(&self.0))
            .map_err(EncodeBech32AddressError::FormatError)
    }
}
