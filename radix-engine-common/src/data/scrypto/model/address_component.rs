use crate::address::{AddressDisplayContext, AddressError, EntityType, NO_NETWORK};
use crate::crypto::{hash, PublicKey};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::well_known_scrypto_custom_type;
use crate::*;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_constants::NODE_ID_LENGTH;
use sbor::rust::fmt;
use sbor::rust::string::String;
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

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }

    pub fn virtual_account_from_public_key<P: Into<PublicKey> + Clone>(
        public_key: &P,
    ) -> ComponentAddress {
        match public_key.clone().into() {
            PublicKey::EcdsaSecp256k1(public_key) => {
                ComponentAddress::EcdsaSecp256k1VirtualAccount(
                    hash(public_key.to_vec()).lower_26_bytes(),
                )
            }
            PublicKey::EddsaEd25519(public_key) => ComponentAddress::EddsaEd25519VirtualAccount(
                hash(public_key.to_vec()).lower_26_bytes(),
            ),
        }
    }

    pub fn virtual_identity_from_public_key<P: Into<PublicKey> + Clone>(
        public_key: &P,
    ) -> ComponentAddress {
        match public_key.clone().into() {
            PublicKey::EcdsaSecp256k1(public_key) => {
                ComponentAddress::EcdsaSecp256k1VirtualIdentity(
                    hash(public_key.to_vec()).lower_26_bytes(),
                )
            }
            PublicKey::EddsaEd25519(public_key) => ComponentAddress::EddsaEd25519VirtualIdentity(
                hash(public_key.to_vec()).lower_26_bytes(),
            ),
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
            NODE_ID_LENGTH => match EntityType::try_from(slice[0])
                .map_err(|_| ParseComponentAddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::NormalComponent
                | EntityType::AccountComponent
                | EntityType::IdentityComponent
                | EntityType::Clock
                | EntityType::EpochManager
                | EntityType::Validator
                | EntityType::EcdsaSecp256k1VirtualAccountComponent
                | EntityType::EddsaEd25519VirtualAccountComponent
                | EntityType::EddsaEd25519VirtualIdentityComponent
                | EntityType::EcdsaSecp256k1VirtualIdentityComponent
                | EntityType::AccessControllerComponent
                | EntityType::FungibleResource
                | EntityType::NonFungibleResource
                | EntityType::Package => {
                    Err(ParseComponentAddressError::InvalidEntityTypeId(slice[0]))
                }
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
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_component_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ComponentAddress::Normal(_) => {
                write!(f, "NormalComponent[{}]", self.to_hex())
            }
            ComponentAddress::Account(_) => {
                write!(f, "AccountComponent[{}]", self.to_hex())
            }
            ComponentAddress::Identity(_) => {
                write!(f, "IdentityComponent[{}]", self.to_hex())
            }
            ComponentAddress::Clock(_) => {
                write!(f, "ClockComponent[{}]", self.to_hex())
            }
            ComponentAddress::EpochManager(_) => {
                write!(f, "EpochManagerComponent[{}]", self.to_hex())
            }
            ComponentAddress::Validator(_) => {
                write!(f, "ValidatorComponent[{}]", self.to_hex())
            }
            ComponentAddress::EcdsaSecp256k1VirtualAccount(_) => {
                write!(
                    f,
                    "EcdsaSecp256k1VirtualAccountComponent[{}]",
                    self.to_hex()
                )
            }
            ComponentAddress::EddsaEd25519VirtualAccount(_) => {
                write!(f, "EddsaEd25519VirtualAccountComponent[{}]", self.to_hex())
            }
            ComponentAddress::AccessController(_) => {
                write!(f, "AccessControllerComponent[{}]", self.to_hex())
            }
            ComponentAddress::EcdsaSecp256k1VirtualIdentity(_) => {
                write!(
                    f,
                    "EcdsaSecp256k1VirtualIdentityComponent[{}]",
                    self.to_hex()
                )
            }
            ComponentAddress::EddsaEd25519VirtualIdentity(_) => {
                write!(f, "EddsaEd25519VirtualIdentityComponent[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
