use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::collections::*;
use sbor::rust::ops::Deref;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::model::*;

#[derive(Debug)]
pub enum StoredValue {
    KeyValueStore(KeyValueStoreId, PreCommittedKeyValueStore),
    Vault(VaultId, Vault),
}

#[derive(Debug)]
pub struct PreCommittedKeyValueStore {
    pub store: HashMap<Vec<u8>, ScryptoValue>,
    pub child_values: HashMap<StoredValueId, RefCell<StoredValue>>,
}

impl PreCommittedKeyValueStore {
    pub fn new() -> Self {
        PreCommittedKeyValueStore {
            store: HashMap::new(),
            child_values: HashMap::new(),
        }
    }

    pub fn all_descendants(&self) -> Vec<StoredValueId> {
        let mut descendents = Vec::new();
        for (id, value) in &self.child_values {
            descendents.push(*id);
            let value = value.borrow();
            if let StoredValue::KeyValueStore(_, store) = value.deref() {
                descendents.extend(store.all_descendants());
            }
        }
        descendents
    }

    pub fn get_child(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> RefMut<StoredValue> {
        if ancestors.is_empty() {
            let value = self
                .child_values
                .get_mut(id)
                .expect("Value expected to exist");
            return value.borrow_mut();
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self
            .child_values
            .get_mut(&StoredValueId::KeyValueStoreId(*first))
            .unwrap();
        match value.get_mut() {
            StoredValue::KeyValueStore(_, store) => store.get_child(rest, id),
            _ => panic!("Expected to be store"),
        }
    }

    pub fn insert_children(&mut self, values: Vec<StoredValue>) {
        for value in values {
            let id = match &value {
                StoredValue::KeyValueStore(id, _) => StoredValueId::KeyValueStoreId(*id),
                StoredValue::Vault(id, _) => StoredValueId::VaultId(*id),
            };
            self.child_values.insert(id, RefCell::new(value));
        }
    }
}
