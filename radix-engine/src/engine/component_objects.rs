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
        self_id: KeyValueStoreId,
        vault_id: &VaultId,
    ) -> Option<(KeyValueStoreId, Vault)> {
        let maybe_vault = self.child_values.remove(&StoredValueId::VaultId(*vault_id));
        if let Some(StoredValue::Vault(_, vault)) = maybe_vault {
            return Option::Some((self_id, vault));
        }

        for child_value in self.child_values.iter_mut() {
            if let (_, StoredValue::KeyValueStore(ref id, kv_store)) = child_value {
                let maybe_vault = kv_store.take_child_vault(*id, vault_id);
                if let Some(vault) = maybe_vault {
                    return Option::Some(vault);
                }
            }
        }

        None
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

    fn get_kv_store_mut_internal(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        // TODO: Optimize to prevent search
        for (_, value) in self.values.iter_mut() {
            if let StoredValue::KeyValueStore(ref id, unclaimed) = value {
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

    pub fn get_kv_store_mut(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&mut PreCommittedKeyValueStore> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        self.get_kv_store_mut_internal(kv_store_id)
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
            if let StoredValue::KeyValueStore(kv_store_id, store) = value {
                let maybe_vault = store.take_child_vault(*kv_store_id, vault_id);
                if let Some((kv_store_id, vault)) = maybe_vault {
                    self.borrowed_vault = Some((*vault_id, Some(kv_store_id)));
                    return Some(vault);
                }
            }
        }

        None
    }

    pub fn return_borrowed_vault_mut(&mut self, vault: Vault) {
        if let Some((vault_id, maybe_parent)) = self.borrowed_vault.take() {
            if let Some(parent_id) = maybe_parent {
                let kv_store = self.get_kv_store_mut_internal(&parent_id).unwrap();
                kv_store.child_values.insert(
                    StoredValueId::VaultId(vault_id.clone()),
                    StoredValue::Vault(vault_id.clone(), vault),
                );
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
