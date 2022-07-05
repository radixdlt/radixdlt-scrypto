/// A unique identifier used in the addressing of Resource Addresses.
const RESOURCE_ADDRESS_ENTITY_ID: u8 = 0x00;

/// A unique identifier used in the addressing of Package Addresses.
const PACKAGE_ADDRESS_ENTITY_ID: u8 = 0x01;

/// A unique identifier used in the addressing of Generic Component Addresses.
const COMPONENT_ADDRESS_ENTITY_ID: u8 = 0x02;

/// A unique identifier used in the addressing of Account Addresses.
const ACCOUNT_ADDRESS_ENTITY_ID: u8 = 0x03;

/// A unique identifier used in the addressing of Account Addresses.
const SYSTEM_ADDRESS_ENTITY_ID: u8 = 0x04;

/// An enum which represents the different addressable entities.
pub enum EntityType {
    Resource,
    Package,
    Component,
    AccountComponent,
    SystemComponent,
}

impl EntityType {
    pub fn id(&self) -> u8 {
        match self {
            Self::Resource => RESOURCE_ADDRESS_ENTITY_ID,
            Self::Package => PACKAGE_ADDRESS_ENTITY_ID,
            Self::Component => COMPONENT_ADDRESS_ENTITY_ID,
            Self::AccountComponent => ACCOUNT_ADDRESS_ENTITY_ID,
            Self::SystemComponent => SYSTEM_ADDRESS_ENTITY_ID,
        }
    }
}

impl TryFrom<u8> for EntityType {
    type Error = EntityTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            RESOURCE_ADDRESS_ENTITY_ID => Ok(Self::Resource),
            PACKAGE_ADDRESS_ENTITY_ID => Ok(Self::Package),
            COMPONENT_ADDRESS_ENTITY_ID => Ok(Self::Component),
            ACCOUNT_ADDRESS_ENTITY_ID => Ok(Self::AccountComponent),
            SYSTEM_ADDRESS_ENTITY_ID => Ok(Self::SystemComponent),
            _ => Err(EntityTypeError::InvalidEntityTypeId(value)),
        }
    }
}

pub enum EntityTypeError {
    InvalidEntityTypeId(u8),
}
