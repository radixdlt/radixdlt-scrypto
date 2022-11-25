use crate::Describe;
use sbor::*;
use super::NonFungibleIdType;

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { divisibility: u8 },

    /// Represents a non-fungible resource
    NonFungible { id_type: NonFungibleIdType },
}

impl ResourceType {
    pub fn divisibility(&self) -> u8 {
        match self {
            ResourceType::Fungible { divisibility } => *divisibility,
            ResourceType::NonFungible { .. } => 0,
        }
    }

    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            ResourceType::Fungible { .. } => panic!("Called id_type on Fungible resource."),
            ResourceType::NonFungible { id_type } => id_type.clone(),
        }
    }
}
