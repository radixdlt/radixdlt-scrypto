use sbor::{Decode, Describe, Encode, TypeId};

use crate::rust::collections::BTreeMap;
use crate::types::*;

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { granularity: u8 },

    /// Represents a non-fungible resource
    NonFungible,
}

/// Represents som supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum NewSupply {
    /// A supply of fungible resource represented by amount.
    Fungible { amount: Decimal },

    /// A supply of non-fungible resource represented by a collection of NFTs keyed by ID.
    NonFungible { entries: BTreeMap<u128, Vec<u8>> },
}

/// Represents the authorization configuration of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub struct ResourceAuthConfigs {
    /// Badge for resource minting.
    pub mint_badge: Address,
    /// Badge for resource updating.
    pub update_badge: Address,
}

impl ResourceAuthConfigs {
    /// Creates a new resource authorization configuration, with one badge for all permissions.
    pub fn new<A: Into<Address>>(root: A) -> Self {
        let address = root.into();
        Self {
            mint_badge: address,
            update_badge: address,
        }
    }

    /// Specifies the mint/burn badge address.
    pub fn with_mint_badge_address<A: Into<Address>>(mut self, address: A) -> Self {
        self.mint_badge = address.into();
        self
    }

    /// Specifies the update badge address.
    pub fn with_update_badge_address<A: Into<Address>>(mut self, address: A) -> Self {
        self.update_badge = address.into();
        self
    }
}
