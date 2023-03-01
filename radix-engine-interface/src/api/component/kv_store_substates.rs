use crate::api::types::{IndexedScryptoValue, RENodeId};
use crate::data::scrypto::ScryptoValue;
use crate::*;
use sbor::rust::collections::*;

// TODO: Josh is leaning towards keeping `Entry::Key` as part of the substate key.
// We will change this implementation if that is agreed.
#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum KeyValueStoreEntrySubstate {
    Some(ScryptoValue, ScryptoValue),
    None,
}

impl KeyValueStoreEntrySubstate {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn owned_node_ids(&self) -> Vec<RENodeId> {
        match self {
            KeyValueStoreEntrySubstate::Some(k, v) => {
                let (_, _, mut owns1, _) = IndexedScryptoValue::from_value(k.clone()).unpack();
                let (_, _, owns2, _) = IndexedScryptoValue::from_value(v.clone()).unpack();
                owns1.extend(owns2);
                owns1
            }
            KeyValueStoreEntrySubstate::None => Vec::new(),
        }
    }

    pub fn global_references(&self) -> HashSet<RENodeId> {
        match self {
            KeyValueStoreEntrySubstate::Some(k, v) => {
                let (_, _, _, mut refs1) = IndexedScryptoValue::from_value(k.clone()).unpack();
                let (_, _, _, refs2) = IndexedScryptoValue::from_value(v.clone()).unpack();
                refs1.extend(refs2);
                refs1
            }
            KeyValueStoreEntrySubstate::None => HashSet::new(),
        }
    }
}
