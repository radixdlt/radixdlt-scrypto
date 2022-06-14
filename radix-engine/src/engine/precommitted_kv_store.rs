use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::ops::{Deref};
use sbor::rust::collections::*;
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

    pub fn get_child_kv_store(
        &mut self,
        ancestors: &[KeyValueStoreId],
        kv_store_id: &KeyValueStoreId,
    ) -> &mut PreCommittedKeyValueStore {
        if ancestors.is_empty() {
            let celled_value = self
                .child_values
                .get_mut(&StoredValueId::KeyValueStoreId(*kv_store_id))
                .expect("Vault expected to exist");
            let value = celled_value.get_mut();
            let store = match value {
                StoredValue::KeyValueStore(_, store) => store,
                _ => panic!("Expected to be a store"),
            };
            return store;
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self
            .child_values
            .get_mut(&StoredValueId::KeyValueStoreId(*first))
            .unwrap();
        let store = match value.get_mut() {
            StoredValue::KeyValueStore(_, store) => store,
            _ => panic!("Expected to be store"),
        };
        store.get_child_kv_store(rest, kv_store_id)
    }

    pub fn take_child_vault(&mut self, ancestors: &[KeyValueStoreId], vault_id: &VaultId) -> RefMut<StoredValue> {
        if ancestors.is_empty() {
            let value = self
                .child_values
                .get_mut(&StoredValueId::VaultId(*vault_id))
                .expect("Vault expected to exist");
            let borrowed = value.borrow_mut();
            return borrowed;
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self
            .child_values
            .get_mut(&StoredValueId::KeyValueStoreId(*first))
            .unwrap();
        match value.get_mut() {
            StoredValue::KeyValueStore(_, store) => store.take_child_vault(rest, vault_id),
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
