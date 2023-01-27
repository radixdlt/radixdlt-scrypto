use super::NonFungibleIdType;
use crate::*;
use sbor::*;

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, Categorize, Encode, Decode, LegacyDescribe, Eq, PartialEq)]
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
}
