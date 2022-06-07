use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::*;

#[derive(Debug)]
pub struct FloatingKeyValueStore {
    pub store: HashMap<Vec<u8>, ScryptoValue>,
    pub child_kv_stores: HashMap<KeyValueStoreId, FloatingKeyValueStore>,
    pub child_vaults: HashMap<VaultId, Vault>,
}

impl FloatingKeyValueStore {
    pub fn new() -> Self {
        FloatingKeyValueStore {
            store: HashMap::new(),
            child_kv_stores: HashMap::new(),
            child_vaults: HashMap::new(),
        }
    }

    fn find_child_kv_store(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut FloatingKeyValueStore> {
        for (id, child_kv_store) in self.child_kv_stores.iter_mut() {
            if id.eq(kv_store_id) {
                return Some(child_kv_store);
            }

            let maybe_store = child_kv_store.find_child_kv_store(kv_store_id);
            if maybe_store.is_some() {
                return maybe_store;
            }
        }

        None
    }

    fn insert_vault(&mut self, vault_id: VaultId, vault: Vault) {
        if self.child_vaults.contains_key(&vault_id) {
            panic!("duplicate vault insertion: {:?}", vault_id);
        }

        self.child_vaults.insert(vault_id, vault);
    }

    fn insert_kv_store(&mut self, kv_store_id: KeyValueStoreId, kv_store: FloatingKeyValueStore) {
        if self.child_kv_stores.contains_key(&kv_store_id) {
            panic!("duplicate store insertion: {:?}", kv_store_id);
        }

        self.child_kv_stores.insert(kv_store_id, kv_store);
    }

    pub fn insert_children(&mut self, values: Vec<StoredValue>) {
        for value in values {
            match value {
                StoredValue::UnclaimedKeyValueStore(id, kv_store) => {
                    self.insert_kv_store(id, kv_store);
                }
                StoredValue::Vault(id, vault) => {
                    self.insert_vault(id, vault);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum StoredValue {
    UnclaimedKeyValueStore(KeyValueStoreId, FloatingKeyValueStore),
    Vault(VaultId, Vault),
}

/// Component type objects which will eventually move into a component
#[derive(Debug)]
pub struct ComponentObjects {
    values: HashMap<StoredValueId, StoredValue>,
    borrowed_vault: Option<(VaultId, Option<KeyValueStoreId>)>,
}

impl ComponentObjects {
    pub fn new() -> Self {
        ComponentObjects {
            values: HashMap::new(),
            borrowed_vault: None,
        }
    }

    pub fn take_all(&mut self) -> HashMap<StoredValueId, StoredValue> {
        self.values.drain().collect()
    }

    pub fn insert(&mut self, id: StoredValueId, value: StoredValue) {
        self.values.insert(id, value);
    }

    pub fn take(
        &mut self,
        other: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

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

    pub fn get_kv_store_entry(
        &mut self,
        kv_store_id: &KeyValueStoreId,
        key: &[u8],
    ) -> Option<Option<ScryptoValue>> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        self.get_kv_store_mut(kv_store_id)
            .map(|kv_store| kv_store.store.get(key).map(|v| v.clone()))
    }

    pub fn get_kv_store_mut(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut FloatingKeyValueStore> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        // TODO: Optimize to prevent search
        for (_, value) in self.values.iter_mut() {
            if let StoredValue::UnclaimedKeyValueStore(ref id, unclaimed) = value {
                if id.eq(kv_store_id) {
                    return Some(unclaimed);
                }

                let maybe_store = unclaimed.find_child_kv_store(kv_store_id);
                if maybe_store.is_some() {
                    return maybe_store;
                }
            }
        }

        None
    }

    pub fn borrow_vault_mut(&mut self, vault_id: &VaultId) -> Option<Vault> {
        if let Some(_) = self.borrowed_vault {
            panic!("Should not be able to borrow multiple times");
        }

        if let Some(vault) = self.values.remove(&StoredValueId::VaultId(*vault_id)) {
            self.borrowed_vault = Some((*vault_id, None));
            match vault {
                StoredValue::Vault(_, vault) => return Some(vault),
                _ => panic!("Expected vault but was {:?}", vault),
            }
        }

        for (_, value) in self.values.iter_mut() {
            if let StoredValue::UnclaimedKeyValueStore(kv_store_id, unclaimed) = value {
                if let Some(vault) = unclaimed.child_vaults.remove(vault_id) {
                    self.borrowed_vault = Some((*vault_id, Some(*kv_store_id)));
                    return Some(vault);
                }
            }
        }

        None
    }

    pub fn return_borrowed_vault_mut(&mut self, vault: Vault) {
        if let Some((vault_id, maybe_ancestor)) = self.borrowed_vault.take() {
            if let Some(ancestor_id) = maybe_ancestor {
                let value = self
                    .values
                    .get_mut(&StoredValueId::KeyValueStoreId(ancestor_id))
                    .unwrap();
                match value {
                    StoredValue::UnclaimedKeyValueStore(_, unclaimed) => {
                        unclaimed.child_vaults.insert(vault_id, vault);
                    }
                    _ => panic!("Expected kv store but was {:?}", value),
                };
            } else {
                self.values.insert(
                    StoredValueId::VaultId(vault_id.clone()),
                    StoredValue::Vault(vault_id.clone(), vault),
                );
            }
        } else {
            panic!("Should never get here");
        }
    }
}
