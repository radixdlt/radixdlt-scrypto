use scrypto::engine::types::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::model::*;

#[derive(Debug)]
pub struct UnclaimedLazyMap {
    pub lazy_map: HashMap<Vec<u8>, Vec<u8>>,
    /// All descendents (not just direct children) of the unclaimed lazy map
    pub descendent_lazy_maps: HashMap<LazyMapId, HashMap<Vec<u8>, Vec<u8>>>,
    pub descendent_vaults: HashMap<VaultId, Vault>,
}

impl UnclaimedLazyMap {
    pub fn new() -> Self {
        UnclaimedLazyMap {
            lazy_map: HashMap::new(),
            descendent_lazy_maps: HashMap::new(),
            descendent_vaults: HashMap::new(),
        }
    }

    fn insert_vault(&mut self, vault_id: VaultId, vault: Vault) {
        if self.descendent_vaults.contains_key(&vault_id) {
            panic!("duplicate vault insertion: {:?}", vault_id);
        }

        self.descendent_vaults.insert(vault_id, vault);
    }

    fn insert_lazy_map(&mut self, lazy_map_id: LazyMapId, lazy_map: HashMap<Vec<u8>, Vec<u8>>) {
        if self.descendent_lazy_maps.contains_key(&lazy_map_id) {
            panic!("duplicate map insertion: {:?}", lazy_map_id);
        }

        self.descendent_lazy_maps.insert(lazy_map_id, lazy_map);
    }

    fn insert_map_descendent(
        &mut self,
        unclaimed_lazy_map: UnclaimedLazyMap,
        lazy_map_id: LazyMapId,
    ) {
        self.insert_lazy_map(lazy_map_id, unclaimed_lazy_map.lazy_map);

        for (lazy_map_id, lazy_map) in unclaimed_lazy_map.descendent_lazy_maps {
            self.insert_lazy_map(lazy_map_id, lazy_map);
        }
        for (vault_id, vault) in unclaimed_lazy_map.descendent_vaults {
            self.insert_vault(vault_id, vault);
        }
    }

    pub fn insert_descendents(&mut self, new_descendents: ComponentObjects) {
        for (vault_id, vault) in new_descendents.vaults {
            self.insert_vault(vault_id, vault);
        }

        for (lazy_map_id, child_lazy_map) in new_descendents.lazy_maps {
            self.insert_map_descendent(child_lazy_map, lazy_map_id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentObjectRefs {
    pub lazy_map_ids: HashSet<LazyMapId>,
    pub vault_ids: HashSet<VaultId>,
}

impl ComponentObjectRefs {
    pub fn new() -> Self {
        ComponentObjectRefs {
            lazy_map_ids: HashSet::new(),
            vault_ids: HashSet::new(),
        }
    }

    pub fn extend(&mut self, other: ComponentObjectRefs) {
        self.lazy_map_ids.extend(other.lazy_map_ids);
        self.vault_ids.extend(other.vault_ids);
    }

    pub fn remove(&mut self, other: &ComponentObjectRefs) -> Result<(), RuntimeError> {
        // Only allow vaults to be added, never removed
        for vault_id in &other.vault_ids {
            if !self.vault_ids.remove(&vault_id) {
                return Err(RuntimeError::VaultRemoved(*vault_id));
            }
        }

        for lazy_map_id in &other.lazy_map_ids {
            if !self.lazy_map_ids.remove(&lazy_map_id) {
                return Err(RuntimeError::LazyMapRemoved(*lazy_map_id));
            }
        }

        Ok(())
    }
}

/// Component type objects which will eventually move into a component
#[derive(Debug)]
pub struct ComponentObjects {
    /// Lazy maps which haven't been assigned to a component or lazy map yet.
    /// Keeps track of vault and lazy map descendents.
    pub lazy_maps: HashMap<LazyMapId, UnclaimedLazyMap>,
    /// Vaults which haven't been assigned to a component or lazy map yet.
    pub vaults: HashMap<VaultId, Vault>,
    borrowed_vault: Option<(VaultId, Option<LazyMapId>)>,
}

impl ComponentObjects {
    pub fn new() -> Self {
        ComponentObjects {
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
            borrowed_vault: None,
        }
    }

    pub fn take(&mut self, other: ComponentObjectRefs) -> Result<ComponentObjects, RuntimeError> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        let mut vaults = HashMap::new();
        let mut lazy_maps = HashMap::new();

        for vault_id in other.vault_ids {
            let vault = self
                .vaults
                .remove(&vault_id)
                .ok_or(RuntimeError::VaultNotFound(vault_id))?;
            vaults.insert(vault_id, vault);
        }

        for lazy_map_id in other.lazy_map_ids {
            let lazy_map = self
                .lazy_maps
                .remove(&lazy_map_id)
                .ok_or(RuntimeError::LazyMapNotFound(lazy_map_id))?;
            lazy_maps.insert(lazy_map_id, lazy_map);
        }

        Ok(ComponentObjects {
            vaults,
            lazy_maps,
            borrowed_vault: None,
        })
    }

    pub fn insert_objects_into_map(
        &mut self,
        new_objects: ComponentObjects,
        lazy_map_id: &LazyMapId,
    ) {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        let unclaimed_map = self.lazy_maps.get_mut(lazy_map_id).unwrap();
        unclaimed_map.insert_descendents(new_objects);
    }

    pub fn insert_lazy_map_entry(&mut self, lazy_map_id: &LazyMapId, key: Vec<u8>, value: Vec<u8>) {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        let (_, lazy_map) = self.get_lazy_map_mut(lazy_map_id).unwrap();
        lazy_map.insert(key, value);
    }

    pub fn get_lazy_map_entry(
        &mut self,
        lazy_map_id: &LazyMapId,
        key: &[u8],
    ) -> Option<(LazyMapId, Option<Vec<u8>>)> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        self.get_lazy_map_mut(lazy_map_id)
            .map(|(lazy_map_id, lazy_map)| (lazy_map_id, lazy_map.get(key).map(|v| v.to_vec())))
    }

    fn get_lazy_map_mut(
        &mut self,
        lazy_map_id: &LazyMapId,
    ) -> Option<(LazyMapId, &mut HashMap<Vec<u8>, Vec<u8>>)> {
        if self.borrowed_vault.is_some() {
            panic!("Should not be taking while value is being borrowed");
        }

        // TODO: Optimize to prevent iteration
        for (root, unclaimed) in self.lazy_maps.iter_mut() {
            if lazy_map_id.eq(root) {
                return Some((root.clone(), &mut unclaimed.lazy_map));
            }

            let lazy_map = unclaimed.descendent_lazy_maps.get_mut(lazy_map_id);
            if lazy_map.is_some() {
                return Some((root.clone(), lazy_map.unwrap()));
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

        for (lazy_map_id, unclaimed) in self.lazy_maps.iter_mut() {
            if let Some(vault) = unclaimed.descendent_vaults.remove(vault_id) {
                self.borrowed_vault = Some((*vault_id, Some(*lazy_map_id)));
                return Some(vault);
            }
        }

        None
    }

    pub fn return_borrowed_vault_mut(&mut self, vault: Vault) {
        if let Some((vault_id, maybe_ancestor)) = self.borrowed_vault.take() {
            if let Some(ancestor_id) = maybe_ancestor {
                self.lazy_maps
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
