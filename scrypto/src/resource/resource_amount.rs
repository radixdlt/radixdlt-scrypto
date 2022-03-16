use crate::math::Decimal;
use crate::resource::*;
use crate::rust::collections::BTreeSet;
use sbor::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Amount {
    /// Fungible amount is represented by some quantity.
    Fungible { amount: Decimal },
    /// Non-fungible amount is represented by a set of IDs.
    NonFungible { ids: BTreeSet<NonFungibleId> },
}

impl Amount {
    /// Treats as fungible by returning a quantity.
    pub fn as_quantity(&self) -> Decimal {
        match self {
            Amount::Fungible { amount } => *amount,
            Amount::NonFungible { ids } => ids.len().into(),
        }
    }

    /// Treats as non-fungible by returning the IDs, if possible.
    pub fn as_non_fungible_ids(&self) -> Option<BTreeSet<NonFungibleId>> {
        match self {
            Amount::Fungible { .. } => None,
            Amount::NonFungible { ids } => Some(ids.clone()),
        }
    }
}
