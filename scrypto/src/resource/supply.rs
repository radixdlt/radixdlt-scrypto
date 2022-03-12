use sbor::*;

use crate::math::*;
use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::vec::Vec;

/// Represents some supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum Supply {
    /// A supply of fungible resources, represented by amount
    Fungible { amount: Decimal },

    /// A supply of non-fungible resources, represented by non-fungible id and data pairs
    NonFungible {
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
    },
}

impl Supply {
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
}
