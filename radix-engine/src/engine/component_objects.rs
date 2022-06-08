use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::*;

#[derive(Debug)]
pub enum StoredValue {
    KeyValueStore(KeyValueStoreId, PreCommittedKeyValueStore),
    Vault(VaultId, Vault),
}

#[derive(Debug)]
pub struct PreCommittedKeyValueStore {
    pub store: HashMap<Vec<u8>, ScryptoValue>,
    pub child_values: HashMap<StoredValueId, StoredValue>,
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
            if let StoredValue::KeyValueStore(_, store) = value {
                descendents.extend(store.all_descendants());
            }
        }
        descendents
    }

    fn find_child_kv_store(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        for (_, child_value) in self.child_values.iter_mut() {
            if let StoredValue::KeyValueStore(ref id, kv_store) = child_value {
                if id.eq(kv_store_id) {
                    return Some(kv_store);
                }

                let maybe_store = kv_store.find_child_kv_store(kv_store_id);
                if maybe_store.is_some() {
                    return maybe_store;
                }
            }
        }

        None
    }

    fn take_child_vault(
        &mut self,
        ancestors: &[KeyValueStoreId],
        vault_id: &VaultId,
    ) -> Vault {
        if ancestors.is_empty() {
            let value = self.child_values.remove(&StoredValueId::VaultId(*vault_id)).expect("Vault expected to exist");
            let vault = match value {
                StoredValue::Vault(_, vault) => vault,
                _ => panic!("Expected to be a vault"),
            };
            return vault;
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self.child_values.get_mut(&StoredValueId::KeyValueStoreId(*first)).unwrap();
        let store = match value {
            StoredValue::KeyValueStore(_, store) => store,
            _ => panic!("Expected to be store"),
        };

        store.take_child_vault(rest, vault_id)
    }

    fn put_child_vault(
        &mut self,
        ancestors: &[KeyValueStoreId],
        vault_id: VaultId,
        vault: Vault
    ) {
        if ancestors.is_empty() {
            self.child_values.insert(StoredValueId::VaultId(vault_id.clone()), StoredValue::Vault(vault_id, vault));
        } else {
            let (first, rest) = ancestors.split_first().unwrap();
            let value = self.child_values.get_mut(&StoredValueId::KeyValueStoreId(*first)).unwrap();
            let store = match value {
                StoredValue::KeyValueStore(_, store) => store,
                _ => panic!("Expected to be store"),
            };
            store.put_child_vault(rest, vault_id, vault);
        }
    }

    pub fn insert_children(&mut self, values: Vec<StoredValue>) {
        for value in values {
            let id = match &value {
                StoredValue::KeyValueStore(id, _) => StoredValueId::KeyValueStoreId(*id),
                StoredValue::Vault(id, _) => StoredValueId::VaultId(*id),
            };
            self.child_values.insert(id, value);
        }
    }
}

/// Component type objects which will eventually move into a component
#[derive(Debug)]
pub struct ComponentObjects {
    pub values: HashMap<StoredValueId, StoredValue>,
}

impl ComponentObjects {
    pub fn new() -> Self {
        ComponentObjects {
            values: HashMap::new(),
        }
    }

    pub fn take_all(&mut self) -> HashMap<StoredValueId, StoredValue> {
        self.values.drain().collect()
    }

    pub fn insert(&mut self, id: StoredValueId, value: StoredValue) {
        self.values.insert(id, value);
    }

    pub fn take(&mut self, id: &StoredValueId) -> Option<StoredValue> {
        self.values.remove(id)
    }

    pub fn take_set(
        &mut self,
        other: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let mut taken_values = Vec::new();

        for id in other {
            let value = self
                .values
                .remove(id)
                .ok_or(RuntimeError::ValueNotFound(*id))?;
            taken_values.push(value);
        }

        Ok(taken_values)
    }

    fn get_owned_kv_store_mut_internal(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        self.values.get_mut(&StoredValueId::KeyValueStoreId(*kv_store_id))
            .map(|v| {
                match v {
                    StoredValue::KeyValueStore(_, store) => store,
                    _ => panic!("Expected KV store")
                }
            })
    }

    fn get_ref_kv_store_mut_internal(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        // TODO: Optimize to prevent search
        for (_, value) in self.values.iter_mut() {
            if let StoredValue::KeyValueStore(_, unclaimed) = value {
                let maybe_store = unclaimed.find_child_kv_store(kv_store_id);
                if maybe_store.is_some() {
                    return maybe_store;
                }
            }
        }

        None
    }

    pub fn get_owned_kv_store_mut(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        self.get_owned_kv_store_mut_internal(kv_store_id)
    }

    pub fn get_ref_kv_store_mut(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        self.get_ref_kv_store_mut_internal(kv_store_id)
    }

    pub fn borrow_ref_vault_mut(&mut self, ancestors: &[KeyValueStoreId], vault_id: &VaultId) -> Vault {
        let (first, rest) = ancestors.split_first().unwrap();
        let store = match self.values.get_mut(&StoredValueId::KeyValueStoreId(*first)).unwrap() {
            StoredValue::KeyValueStore(_, store) => store,
            _ => panic!("Should not get here"),
        };
        store.take_child_vault(rest, vault_id)
    }

    pub fn return_borrowed_vault_mut(&mut self, ancestors: &[KeyValueStoreId], vault_id: VaultId, vault: Vault) {
        let (first, rest) = ancestors.split_first().unwrap();
        let store = match self.values.get_mut(&StoredValueId::KeyValueStoreId(*first)).unwrap() {
            StoredValue::KeyValueStore(_, store) => store,
            _ => panic!("Should not get here"),
        };
        store.put_child_vault(rest, vault_id, vault);
    }
}
