use crate::api::types::{IndexedScryptoValue, RENodeId};
use crate::data::scrypto::ScryptoValue;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum KeyValueStoreEntrySubstate {
    Some(ScryptoValue),
    None,
}

impl KeyValueStoreEntrySubstate {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn owned_node_ids(&self) -> Vec<RENodeId> {
        match self {
            KeyValueStoreEntrySubstate::Some(v) => {
                IndexedScryptoValue::from_scrypto_value(v.clone())
                    .unpack()
                    .1
            }
            KeyValueStoreEntrySubstate::None => Vec::new(),
        }
    }

    pub fn references(&self) -> HashSet<RENodeId> {
        match self {
            KeyValueStoreEntrySubstate::Some(v) => {
                IndexedScryptoValue::from_scrypto_value(v.clone())
                    .unpack()
                    .2
            }
            KeyValueStoreEntrySubstate::None => HashSet::new(),
        }
    }
}
