use crate::component::{ComponentAddress, PackageAddress};
use crate::resource::ResourceAddress;

/// A unique identifier used in the addressing of Resource Addresses.
pub const RESOURCE_ADDRESS_ENTITY_ID: u8 = 0x00;

/// A unique identifier used in the addressing of Package Addresses.
pub const PACKAGE_ADDRESS_ENTITY_ID: u8 = 0x01;

/// A unique identifier used in the addressing of Generic Component Addresses.
pub const NORMAL_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x02;

/// A unique identifier used in the addressing of Account Component Addresses.
pub const ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x03;

/// A unique identifier used in the addressing of System Component Addresses.
pub const SYSTEM_COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x04;

/// An enum which represents the different addressable entities.
#[derive(PartialEq, Eq)]
pub enum EntityType {
    Resource,
    Package,
    NormalComponent,
    AccountComponent,
    SystemComponent,
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
            ComponentAddress::System(_) => Self::SystemComponent,
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            Self::Resource => RESOURCE_ADDRESS_ENTITY_ID,
            Self::Package => PACKAGE_ADDRESS_ENTITY_ID,
            Self::NormalComponent => NORMAL_COMPONENT_ADDRESS_ENTITY_ID,
            Self::AccountComponent => ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID,
            Self::SystemComponent => SYSTEM_COMPONENT_ADDRESS_ENTITY_ID,
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
            SYSTEM_COMPONENT_ADDRESS_ENTITY_ID => Ok(Self::SystemComponent),
            _ => Err(EntityTypeError::InvalidEntityTypeId(value)),
        }
    }
}

pub enum EntityTypeError {
    InvalidEntityTypeId(u8),
}
