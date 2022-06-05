use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::*;

#[derive(Debug)]
pub struct UnclaimedKeyValueStore {
    pub kv_store: HashMap<Vec<u8>, ScryptoValue>,
    /// All descendents (not just direct children) of the store
    pub descendent_kv_stores: HashMap<KeyValueStoreId, HashMap<Vec<u8>, ScryptoValue>>,
    pub descendent_vaults: HashMap<VaultId, Vault>,
}

impl UnclaimedKeyValueStore {
    pub fn new() -> Self {
        UnclaimedKeyValueStore {
            kv_store: HashMap::new(),
            descendent_kv_stores: HashMap::new(),
            descendent_vaults: HashMap::new(),
        }
    }

    fn insert_vault(&mut self, vault_id: VaultId, vault: Vault) {
        if self.descendent_vaults.contains_key(&vault_id) {
            panic!("duplicate vault insertion: {:?}", vault_id);
        }

        self.descendent_vaults.insert(vault_id, vault);
    }

    fn insert_kv_store(
        &mut self,
        kv_store_id: KeyValueStoreId,
        kv_store: HashMap<Vec<u8>, ScryptoValue>,
    ) {
        if self.descendent_kv_stores.contains_key(&kv_store_id) {
            panic!("duplicate store insertion: {:?}", kv_store_id);
        }

        self.descendent_kv_stores.insert(kv_store_id, kv_store);
    }

    fn insert_store_descendent(
        &mut self,
        unclaimed_kv_store: UnclaimedKeyValueStore,
        kv_store_id: KeyValueStoreId,
    ) {
        self.insert_kv_store(kv_store_id, unclaimed_kv_store.kv_store);

        for (kv_store_id, kv_store) in unclaimed_kv_store.descendent_kv_stores {
            self.insert_kv_store(kv_store_id, kv_store);
        }
        for (vault_id, vault) in unclaimed_kv_store.descendent_vaults {
            self.insert_vault(vault_id, vault);
        }
    }

    pub fn insert_descendents(&mut self, new_descendents: ComponentObjects) {
        for (vault_id, vault) in new_descendents.vaults {
            self.insert_vault(vault_id, vault);
        }

        for (kv_store_id, child_kv_store) in new_descendents.kv_stores {
            self.insert_store_descendent(child_kv_store, kv_store_id);
        }
    }
}

/// Component type objects which will eventually move into a component
#[derive(Debug)]
pub struct ComponentObjects {
    /// Key/Value stores which haven't been assigned to a component or another store yet.
    /// Keeps track of vault and store descendents.
    pub kv_stores: HashMap<KeyValueStoreId, UnclaimedKeyValueStore>,
    /// Vaults which haven't been assigned to a component or store yet.
    pub vaults: HashMap<VaultId, Vault>,
    borrowed_vault: Option<(VaultId, Option<KeyValueStoreId>)>,
}

impl ComponentObjects {
    pub fn new() -> Self {
        ComponentObjects {
            kv_stores: HashMap::new(),
            vaults: HashMap::new(),
            borrowed_vault: None,
        }
    }

    pub fn take(&mut self, other: HashSet<StoredValueId>) -> Result<ComponentObjects, RuntimeError> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        let mut vaults = HashMap::new();
        let mut kv_stores = HashMap::new();

        for id in other {
            match id {
                StoredValueId::KeyValueStoreId(kv_store_id) => {
                    let kv_store = self
                        .kv_stores
                        .remove(&kv_store_id)
                        .ok_or(RuntimeError::KeyValueStoreNotFound(kv_store_id))?;
                    kv_stores.insert(kv_store_id, kv_store);
                }
                StoredValueId::VaultId(vault_id) => {
                    let vault = self
                        .vaults
                        .remove(&vault_id)
                        .ok_or(RuntimeError::VaultNotFound(vault_id))?;
                    vaults.insert(vault_id, vault);
                }
            }
        }

        Ok(ComponentObjects {
            vaults,
            kv_stores,
            borrowed_vault: None,
        })
    }

    pub fn insert_objects_into_kv_store(
        &mut self,
        new_objects: ComponentObjects,
        kv_store_id: &KeyValueStoreId,
    ) {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        let unclaimed_kv_store = self.kv_stores.get_mut(kv_store_id).unwrap();
        unclaimed_kv_store.insert_descendents(new_objects);
    }

    pub fn insert_kv_store_entry(
        &mut self,
        kv_store_id: &KeyValueStoreId,
        key: Vec<u8>,
        value: ScryptoValue,
    ) {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        let (_, kv_store) = self.get_kv_store_mut(kv_store_id).unwrap();
        kv_store.insert(key, value);
    }

    pub fn get_kv_store_entry(
        &mut self,
        kv_store_id: &KeyValueStoreId,
        key: &[u8],
    ) -> Option<(KeyValueStoreId, Option<ScryptoValue>)> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        self.get_kv_store_mut(kv_store_id)
            .map(|(kv_store_id, kv_store)| (kv_store_id, kv_store.get(key).map(|v| v.clone())))
    }

    fn get_kv_store_mut(
        &mut self,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<(KeyValueStoreId, &mut HashMap<Vec<u8>, ScryptoValue>)> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        // TODO: Optimize to prevent iteration
        for (root, unclaimed) in self.kv_stores.iter_mut() {
            if kv_store_id.eq(root) {
                return Some((root.clone(), &mut unclaimed.kv_store));
            }

            let kv_store = unclaimed.descendent_kv_stores.get_mut(kv_store_id);
            if kv_store.is_some() {
                return Some((root.clone(), kv_store.unwrap()));
            }
        }

        None
    }

    pub fn borrow_vault_mut(&mut self, vault_id: &VaultId) -> Option<Vault> {
        if let Some(_) = self.borrowed_vault {
            panic!("Should not be able to borrow multiple times");
        }

        if let Some(vault) = self.vaults.remove(vault_id) {
            self.borrowed_vault = Some((*vault_id, None));
            return Some(vault);
        }

        for (kv_store_id, unclaimed) in self.kv_stores.iter_mut() {
            if let Some(vault) = unclaimed.descendent_vaults.remove(vault_id) {
                self.borrowed_vault = Some((*vault_id, Some(*kv_store_id)));
                return Some(vault);
            }
        }

        None
    }

    pub fn return_borrowed_vault_mut(&mut self, vault: Vault) {
        if let Some((vault_id, maybe_ancestor)) = self.borrowed_vault.take() {
            if let Some(ancestor_id) = maybe_ancestor {
                self.kv_stores
                    .get_mut(&ancestor_id)
                    .unwrap()
                    .descendent_vaults
                    .insert(vault_id, vault);
            } else {
                self.vaults.insert(vault_id, vault);
            }
        } else {
            panic!("Should never get here");
        }
    }
}
