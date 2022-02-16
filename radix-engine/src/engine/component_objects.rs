use scrypto::rust::collections::*;
use scrypto::types::*;

use crate::model::*;

#[derive(Debug)]
pub struct UnclaimedLazyMap {
    pub lazy_map: LazyMap,
    /// All descendents (not just direct children) of the unclaimed lazy map
    pub descendent_lazy_maps: HashMap<Mid, LazyMap>,
    pub descendent_vaults: HashMap<Vid, Vault>,
}

impl UnclaimedLazyMap {
    fn insert_vault(&mut self, vid: Vid, vault: Vault) {
        if self.descendent_vaults.contains_key(&vid) {
            panic!("duplicate vault insertion: {}", vid);
        }

        self.descendent_vaults.insert(vid, vault);
    }

    fn insert_lazy_map(&mut self, mid: Mid, lazy_map: LazyMap) {
        if self.descendent_lazy_maps.contains_key(&mid) {
            panic!("duplicate map insertion: {}", mid);
        }

        self.descendent_lazy_maps.insert(mid, lazy_map);
    }

    fn insert_map_descendent(&mut self, unclaimed_lazy_map: UnclaimedLazyMap, mid: Mid) {
        self.insert_lazy_map(mid, unclaimed_lazy_map.lazy_map);

        for (mid, lazy_map) in unclaimed_lazy_map.descendent_lazy_maps {
            self.insert_lazy_map(mid, lazy_map);
        }
        for (vid, vault) in unclaimed_lazy_map.descendent_vaults {
            self.insert_vault(vid, vault);
        }
    }

    pub fn insert_descendents(&mut self, new_descendents: ComponentObjects) {
        for (vid, vault) in new_descendents.vaults {
            self.insert_vault(vid, vault);
        }

        for (mid, child_lazy_map) in new_descendents.lazy_maps {
            self.insert_map_descendent(child_lazy_map, mid);
        }
    }
}

pub struct ComponentObjectRefs {
    pub mids: HashSet<Mid>,
    pub vids: HashSet<Vid>,
}

impl ComponentObjectRefs {
    pub fn new() -> Self {
        ComponentObjectRefs {
            mids: HashSet::new(),
            vids: HashSet::new(),
        }
    }

    pub fn extend(&mut self, other: ComponentObjectRefs) {
        self.mids.extend(other.mids);
        self.vids.extend(other.vids);
    }

    pub fn remove(&mut self, other: &ComponentObjectRefs) -> Result<(), RuntimeError> {
        // Only allow vaults to be added, never removed
        for vid in &other.vids {
            if !self.vids.remove(&vid) {
                return Err(RuntimeError::VaultRemoved(vid.clone()));
            }
        }

        for mid in &other.mids {
            if !self.mids.remove(&mid) {
                return Err(RuntimeError::LazyMapRemoved(mid.clone()));
            }
        }

        Ok(())
    }
}

/// Component type objects which will eventually move into a component
pub struct ComponentObjects {
    /// Lazy maps which haven't been assigned to a component or lazy map yet.
    /// Keeps track of vault and lazy map descendents.
    pub lazy_maps: HashMap<Mid, UnclaimedLazyMap>,
    /// Vaults which haven't been assigned to a component or lazy map yet.
    pub vaults: HashMap<Vid, Vault>,
}

impl ComponentObjects {
    pub fn new() -> Self {
        ComponentObjects {
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
        }
    }

    pub fn take(&mut self, other: ComponentObjectRefs) -> Result<ComponentObjects, RuntimeError> {
        let mut vaults = HashMap::new();
        let mut lazy_maps = HashMap::new();

        for vid in other.vids {
            let vault = self
                .vaults
                .remove(&vid)
                .ok_or(RuntimeError::VaultNotFound(vid))?;
            vaults.insert(vid, vault);
        }

        for mid in other.mids {
            let lazy_map = self
                .lazy_maps
                .remove(&mid)
                .ok_or(RuntimeError::LazyMapNotFound(mid))?;
            lazy_maps.insert(mid, lazy_map);
        }

        Ok(ComponentObjects { vaults, lazy_maps })
    }

    pub fn insert_objects_into_map(&mut self, new_objects: ComponentObjects, mid: &Mid) {
        let unclaimed_map = self.lazy_maps.get_mut(mid).unwrap();
        unclaimed_map.insert_descendents(new_objects);
    }

    pub fn get_lazy_map_mut(&mut self, mid: &Mid) -> Option<(Mid, &mut LazyMap)> {
        // TODO: Optimize to prevent iteration
        for (root, unclaimed) in self.lazy_maps.iter_mut() {
            if mid.eq(root) {
                return Some((root.clone(), &mut unclaimed.lazy_map));
            }

            let lazy_map = unclaimed.descendent_lazy_maps.get_mut(mid);
            if lazy_map.is_some() {
                return Some((root.clone(), lazy_map.unwrap()));
            }
        }

        None
    }

    pub fn get_vault_mut(&mut self, vid: &Vid) -> Option<&mut Vault> {
        let vault = self.vaults.get_mut(vid);
        if vault.is_some() {
            return Some(vault.unwrap());
        }

        // TODO: Optimize to prevent iteration
        for (_, unclaimed) in self.lazy_maps.iter_mut() {
            let vault = unclaimed.descendent_vaults.get_mut(vid);
            if vault.is_some() {
                return Some(vault.unwrap());
            }
        }

        None
    }
}
