use crate::api::types::{IndexedScryptoValue, RENodeId};
use crate::data::scrypto::ScryptoValue;
use crate::*;
use sbor::rust::prelude::*;

// TODO: Josh is leaning towards keeping `Entry::Key` as part of the substate key.
// We will change this implementation if that is agreed.
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
                let (_, _, own, _) = IndexedScryptoValue::from_value(v.clone()).unpack();
                own
            }
            KeyValueStoreEntrySubstate::None => Vec::new(),
        }
    }

    pub fn global_references(&self) -> HashSet<RENodeId> {
        match self {
            KeyValueStoreEntrySubstate::Some(v) => {
                let (_, _, _, refs) = IndexedScryptoValue::from_value(v.clone()).unpack();
                refs
            }
            KeyValueStoreEntrySubstate::None => HashSet::new(),
        }
    }
}
