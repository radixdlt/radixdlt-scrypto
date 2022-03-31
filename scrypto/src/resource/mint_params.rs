use sbor::*;

use crate::math::*;
use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::vec::Vec;

/// Represents the minting parameters
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum MintParams {
    /// To mint fungible resource, represented by an amount
    Fungible { amount: Decimal },

    /// To mint non-fungible resource, represented by non-fungible id and data pairs
    NonFungible {
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
    },
}

impl MintParams {
    pub fn fungible<T: Into<Decimal>>(amount: T) -> Self {
        Self::Fungible {
            amount: amount.into(),
        }
    }

    pub fn non_fungible<T, V>(entries: T) -> Self
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data(), e.mutable_data()));
        }

        Self::NonFungible { entries: encoded }
    }

    pub fn matches_type(&self, resource_type: &ResourceType) -> bool {
        match self {
            Self::Fungible { .. } => {
                matches!(resource_type, ResourceType::Fungible { .. })
            }
            Self::NonFungible { .. } => matches!(resource_type, ResourceType::NonFungible),
        }
    }

    pub fn amount(&self) -> Decimal {
        match self {
            Self::Fungible { amount } => amount.clone(),
            Self::NonFungible { entries } => entries.len().into(),
        }
    }
}
