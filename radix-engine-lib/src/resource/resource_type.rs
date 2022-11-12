use sbor::*;

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { divisibility: u8 },

    /// Represents a non-fungible resource
    NonFungible,
}

impl ResourceType {
    pub fn divisibility(&self) -> u8 {
        match self {
            ResourceType::Fungible { divisibility } => *divisibility,
            ResourceType::NonFungible => 0,
        }
    }
}
