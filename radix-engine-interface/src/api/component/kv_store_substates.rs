use crate::api::types::*;
use crate::data::IndexedScryptoValue;
use radix_engine_derive::*;
use radix_engine_interface::data::ScryptoValue;
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

    pub fn owned_node_ids(&self) -> HashSet<RENodeId> {
        let mut owned_node_ids = HashSet::new();
        match self {
            KeyValueStoreEntrySubstate::Some(k, v) => {
                let k = IndexedScryptoValue::from_value(k.clone());
                owned_node_ids.extend(k.owned_node_ids().unwrap());
                let v = IndexedScryptoValue::from_value(v.clone());
                owned_node_ids.extend(v.owned_node_ids().unwrap());
            }
            KeyValueStoreEntrySubstate::None => {}
        }
        owned_node_ids
    }

    pub fn global_references(&self) -> HashSet<GlobalAddress> {
        let mut global_references = HashSet::new();
        match self {
            KeyValueStoreEntrySubstate::Some(k, v) => {
                let k = IndexedScryptoValue::from_value(k.clone());
                global_references.extend(k.global_references());
                let v = IndexedScryptoValue::from_value(v.clone());
                global_references.extend(v.global_references());
            }
            KeyValueStoreEntrySubstate::None => {}
        }
        global_references
    }
}
