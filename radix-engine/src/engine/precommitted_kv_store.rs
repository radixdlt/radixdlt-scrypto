use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::collections::hash_map::IntoIter;
use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::model::*;

#[derive(Debug)]
pub struct InMemoryChildren {
    child_values: HashMap<StoredValueId, RefCell<StoredValue>>,
}

impl InMemoryChildren {
    pub fn new() -> Self {
        InMemoryChildren {
            child_values: HashMap::new(),
        }
    }

    pub fn with_values(values: HashMap<StoredValueId, StoredValue>) -> Self {
        let mut child_values = HashMap::new();
        for (id, value) in values.into_iter() {
            child_values.insert(id, RefCell::new(value));
        }
        InMemoryChildren { child_values }
    }

    pub fn into_iter(self) -> IntoIter<StoredValueId, RefCell<StoredValue>> {
        self.child_values.into_iter()
    }

    pub fn all_descendants(&self) -> Vec<StoredValueId> {
        let mut descendents = Vec::new();
        for (id, value) in self.child_values.iter() {
            descendents.push(*id);
            let value = value.borrow();
            descendents.extend(value.all_descendants());
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
        value.get_mut().get_child(rest, id)
    }

    pub fn get_child_mut(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> &mut StoredValue {
        if ancestors.is_empty() {
            let value = self
                .child_values
                .get_mut(id)
                .expect("Value expected to exist");
            return value.get_mut();
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self
            .child_values
            .get_mut(&StoredValueId::KeyValueStoreId(*first))
            .unwrap();
        value.get_mut().get_child_mut(rest, id)
    }

    pub fn insert_children(&mut self, values: HashMap<StoredValueId, StoredValue>) {
        for (id, value) in values {
            self.child_values.insert(id, RefCell::new(value));
        }
    }
}

#[derive(Debug)]
pub enum StoredValue {
    KeyValueStore {
        store: PreCommittedKeyValueStore,
        child_values: InMemoryChildren,
    },
    Vault(Vault),
}

impl StoredValue {
    pub fn kv_store(&self) -> &PreCommittedKeyValueStore {
        match self {
            StoredValue::KeyValueStore { store, .. } => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store_mut(&mut self) -> &mut PreCommittedKeyValueStore {
        match self {
            StoredValue::KeyValueStore { store, .. } => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn vault(&self) -> &Vault {
        match self {
            StoredValue::Vault(vault) => vault,
            _ => panic!("Expected to be a vault"),
        }
    }

    pub fn all_descendants(&self) -> Vec<StoredValueId> {
        match self {
            StoredValue::KeyValueStore { child_values, .. } => child_values.all_descendants(),
            _ => Vec::new(),
        }
    }

    pub fn get_child(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> RefMut<StoredValue> {
        match self {
            StoredValue::KeyValueStore { child_values, .. } => {
                child_values.get_child(ancestors, id)
            }
            _ => panic!("Expected to be store"),
        }
    }

    pub fn get_child_mut(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> &mut StoredValue {
        match self {
            StoredValue::KeyValueStore { child_values, .. } => {
                child_values.get_child_mut(ancestors, id)
            }
            _ => panic!("Expected to be store"),
        }
    }

    pub fn insert_children(&mut self, values: HashMap<StoredValueId, StoredValue>) {
        match self {
            StoredValue::KeyValueStore { child_values, .. } => child_values.insert_children(values),
            _ => panic!("Expected to be store"),
        }
    }
}

#[derive(Debug)]
pub struct PreCommittedKeyValueStore {
    pub store: HashMap<Vec<u8>, ScryptoValue>,
}

impl PreCommittedKeyValueStore {
    pub fn new() -> Self {
        PreCommittedKeyValueStore {
            store: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: ScryptoValue) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<ScryptoValue> {
        self.store.get(key).cloned()
    }
}
