use sbor::{Decode, Describe, Encode, TypeId};

use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::vec::Vec;
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

/// Represents some supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum NewSupply {
    /// A supply of fungible resource represented by amount.
    Fungible { amount: Decimal },

    /// A supply of non-fungible resource represented by a collection of NFTs keyed by ID.
    NonFungible {
        entries: HashMap<NftKey, (Vec<u8>, Vec<u8>)>,
    },
}

impl NewSupply {
    pub fn fungible<T: Into<Decimal>>(amount: T) -> Self {
        Self::Fungible {
            amount: amount.into(),
        }
    }

    pub fn non_fungible<T, V>(entries: T) -> Self
    where
        T: IntoIterator<Item = (NftKey, V)>,
        V: NftData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data(), e.mutable_data()));
        }

        Self::NonFungible { entries: encoded }
    }
}
