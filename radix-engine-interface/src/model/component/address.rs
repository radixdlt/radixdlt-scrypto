use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::{copy_u8_array, ContextualDisplay};

use crate::abi::*;
use crate::address::*;
use crate::crypto::{hash, PublicKey};
use crate::data::ScryptoCustomValueKind;
use crate::scrypto_type;

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentAddress {
    Normal([u8; 26]),
    Account([u8; 26]),
    Identity([u8; 26]),
    Clock([u8; 26]),
    EpochManager([u8; 26]),
    Validator([u8; 26]),
    EcdsaSecp256k1VirtualAccount([u8; 26]),
    EddsaEd25519VirtualAccount([u8; 26]),
    EcdsaSecp256k1VirtualIdentity([u8; 26]),
    EddsaEd25519VirtualIdentity([u8; 26]),
    AccessController([u8; 26]),
}

//========
// binary
//========

impl TryFrom<&[u8]> for ComponentAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::NormalComponent => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                EntityType::AccountComponent => Ok(Self::Account(copy_u8_array(&slice[1..]))),
                EntityType::IdentityComponent => Ok(Self::Identity(copy_u8_array(&slice[1..]))),
                EntityType::Clock => Ok(Self::Clock(copy_u8_array(&slice[1..]))),
                EntityType::EpochManager => Ok(Self::EpochManager(copy_u8_array(&slice[1..]))),
                EntityType::Validator => Ok(Self::Validator(copy_u8_array(&slice[1..]))),
                EntityType::EcdsaSecp256k1VirtualAccountComponent => Ok(
                    Self::EcdsaSecp256k1VirtualAccount(copy_u8_array(&slice[1..])),
                ),
                EntityType::EddsaEd25519VirtualAccountComponent => {
                    Ok(Self::EddsaEd25519VirtualAccount(copy_u8_array(&slice[1..])))
                }
                EntityType::EddsaEd25519VirtualIdentityComponent => Ok(
                    Self::EddsaEd25519VirtualIdentity(copy_u8_array(&slice[1..])),
                ),
                EntityType::AccessControllerComponent => {
                    Ok(Self::AccessController(copy_u8_array(&slice[1..])))
                }
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ComponentAddress {
    pub fn raw(&self) -> [u8; 26] {
        match self {
            Self::Normal(v)
            | Self::Account(v)
            | Self::Clock(v)
            | Self::EpochManager(v)
            | Self::Validator(v)
            | Self::EcdsaSecp256k1VirtualAccount(v)
            | Self::EddsaEd25519VirtualAccount(v)
            | Self::EcdsaSecp256k1VirtualIdentity(v)
            | Self::EddsaEd25519VirtualIdentity(v)
            | Self::Identity(v)
            | Self::AccessController(v) => v.clone(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::component(self).id());
        match self {
            Self::Normal(v)
            | Self::Account(v)
            | Self::Identity(v)
            | Self::Clock(v)
            | Self::EpochManager(v)
            | Self::Validator(v)
            | Self::EddsaEd25519VirtualAccount(v)
            | Self::EcdsaSecp256k1VirtualAccount(v)
            | Self::EcdsaSecp256k1VirtualIdentity(v)
            | Self::EddsaEd25519VirtualIdentity(v)
            | Self::AccessController(v) => buf.extend(v),
        }
        buf
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

scrypto_type!(
    ComponentAddress,
    ScryptoCustomValueKind::ComponentAddress,
    Type::ComponentAddress,
    27
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
