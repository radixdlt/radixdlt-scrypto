use crate::model::*;

/// A unique identifier used in the addressing of Resource Addresses.
pub const RESOURCE_ADDRESS_ENTITY_ID: u8 = 0x00;

/// A unique identifier used in the addressing of Package Addresses.
pub const PACKAGE_ADDRESS_ENTITY_ID: u8 = 0x01;

/// A unique identifier used in the addressing of Generic Component Addresses.
pub const NORMAL_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x02;

/// A unique identifier used in the addressing of Account Component Addresses.
pub const ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x03;

/// A unique identifier used in the addressing of Epoch Manager System Addresses.
pub const EPOCH_MANAGER_SYSTEM_ADDRESS_ENTITY_ID: u8 = 0x04;

/// A unique identifier used in the addressing of Validator System Addresses.
pub const VALIDATOR_SYSTEM_ADDRESS_ENTITY_ID: u8 = 0x05;

/// A unique identifier used in the addressing of Clock System Addresses.
pub const CLOCK_SYSTEM_ADDRESS_ENTITY_ID: u8 = 0x06;

/// A unique identifier used in the addressing of a virtual Account Component Addresses.
pub const ECDSA_SECP_256K1_VIRTUAL_ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x07;

/// A unique identifier used in the addressing of a virtual Account Component Addresses.
pub const EDDSA_ED_25519_VIRTUAL_ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x08;

/// A unique identifier used in the addressing of identity components.
pub const IDENTITY_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x09;

/// A unique identifier used in the addressing of a virtual Account Component Addresses.
pub const ECDSA_SECP_256K1_VIRTUAL_IDENTITY_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x0a;

/// A unique identifier used in the addressing of a virtual Account Component Addresses.
pub const EDDSA_ED_25519_VIRTUAL_IDENTITY_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x0b;

/// A unique identifier used in the addressing of Account Controller Component Addresses.
pub const ACCESS_CONTROLLER_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x0c;

/// An enum which represents the different addressable entities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum EntityType {
    Resource,
    Package,
    NormalComponent,
    AccountComponent,
    IdentityComponent,
    EpochManager,
    Validator,
    Clock,
    EcdsaSecp256k1VirtualAccountComponent,
    EddsaEd25519VirtualAccountComponent,
    EcdsaSecp256k1VirtualIdentityComponent,
    EddsaEd25519VirtualIdentityComponent,
    AccessControllerComponent,
}

impl EntityType {
    pub fn package(_address: &PackageAddress) -> Self {
        Self::Package
    }
    pub fn resource(_address: &ResourceAddress) -> Self {
        Self::Resource
    }
    pub fn component(address: &ComponentAddress) -> Self {
        match address {
            ComponentAddress::Normal(_) => Self::NormalComponent,
            ComponentAddress::Account(_) => Self::AccountComponent,
            ComponentAddress::Identity(_) => Self::IdentityComponent,
            ComponentAddress::Clock(_) => Self::Clock,
            ComponentAddress::EpochManager(_) => Self::EpochManager,
            ComponentAddress::Validator(_) => Self::Validator,
            ComponentAddress::EcdsaSecp256k1VirtualAccount(_) => {
                Self::EcdsaSecp256k1VirtualAccountComponent
            }
            ComponentAddress::EddsaEd25519VirtualAccount(_) => {
                Self::EddsaEd25519VirtualAccountComponent
            }
            ComponentAddress::AccessController(_) => Self::AccessControllerComponent,
            ComponentAddress::EcdsaSecp256k1VirtualIdentity(_) => {
                Self::EcdsaSecp256k1VirtualIdentityComponent
            }
            ComponentAddress::EddsaEd25519VirtualIdentity(_) => {
                Self::EddsaEd25519VirtualIdentityComponent
            }
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            Self::Resource => RESOURCE_ADDRESS_ENTITY_ID,
            Self::Package => PACKAGE_ADDRESS_ENTITY_ID,
            Self::NormalComponent => NORMAL_COMPONENT_ADDRESS_ENTITY_ID,
            Self::AccountComponent => ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID,
            Self::IdentityComponent => IDENTITY_COMPONENT_ADDRESS_ENTITY_ID,
            Self::EpochManager => EPOCH_MANAGER_SYSTEM_ADDRESS_ENTITY_ID,
            Self::Validator => VALIDATOR_SYSTEM_ADDRESS_ENTITY_ID,
            Self::Clock => CLOCK_SYSTEM_ADDRESS_ENTITY_ID,
            Self::EcdsaSecp256k1VirtualAccountComponent => {
                ECDSA_SECP_256K1_VIRTUAL_ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID
            }
            Self::EddsaEd25519VirtualAccountComponent => {
                EDDSA_ED_25519_VIRTUAL_ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID
            }
            Self::AccessControllerComponent => ACCESS_CONTROLLER_COMPONENT_ADDRESS_ENTITY_ID,
            Self::EcdsaSecp256k1VirtualIdentityComponent => {
                ECDSA_SECP_256K1_VIRTUAL_IDENTITY_COMPONENT_ADDRESS_ENTITY_ID
            }
            Self::EddsaEd25519VirtualIdentityComponent => {
                EDDSA_ED_25519_VIRTUAL_IDENTITY_COMPONENT_ADDRESS_ENTITY_ID
            }
        }
    }
}

impl TryFrom<u8> for EntityType {
    type Error = EntityTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            RESOURCE_ADDRESS_ENTITY_ID => Ok(Self::Resource),
            PACKAGE_ADDRESS_ENTITY_ID => Ok(Self::Package),
            NORMAL_COMPONENT_ADDRESS_ENTITY_ID => Ok(Self::NormalComponent),
            ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID => Ok(Self::AccountComponent),
            IDENTITY_COMPONENT_ADDRESS_ENTITY_ID => Ok(Self::IdentityComponent),
            EPOCH_MANAGER_SYSTEM_ADDRESS_ENTITY_ID => Ok(Self::EpochManager),
            VALIDATOR_SYSTEM_ADDRESS_ENTITY_ID => Ok(Self::Validator),
            CLOCK_SYSTEM_ADDRESS_ENTITY_ID => Ok(Self::Clock),
            ECDSA_SECP_256K1_VIRTUAL_ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID => {
                Ok(Self::EcdsaSecp256k1VirtualAccountComponent)
            }
            EDDSA_ED_25519_VIRTUAL_ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID => {
                Ok(Self::EddsaEd25519VirtualAccountComponent)
            }
            ECDSA_SECP_256K1_VIRTUAL_IDENTITY_COMPONENT_ADDRESS_ENTITY_ID => {
                Ok(Self::EcdsaSecp256k1VirtualIdentityComponent)
            }
            EDDSA_ED_25519_VIRTUAL_IDENTITY_COMPONENT_ADDRESS_ENTITY_ID => {
                Ok(Self::EddsaEd25519VirtualIdentityComponent)
            }
            ACCESS_CONTROLLER_COMPONENT_ADDRESS_ENTITY_ID => Ok(Self::AccessControllerComponent),
            _ => Err(EntityTypeError::InvalidEntityTypeId(value)),
        }
    }
}

pub enum EntityTypeError {
    InvalidEntityTypeId(u8),
}
