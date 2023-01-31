use crate::types::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode, PartialEq, Eq)]
pub enum KeyValueStoreEntrySubstate {
    Some(ScryptoValue, ScryptoValue),
    None,
}

impl KeyValueStoreEntrySubstate {
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
