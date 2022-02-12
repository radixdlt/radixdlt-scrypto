use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
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
    pub fn merge(&mut self, unclaimed_lazy_map: UnclaimedLazyMap, mid: Mid) {
        self.descendent_lazy_maps
            .insert(mid, unclaimed_lazy_map.lazy_map);

        for (mid, lazy_map) in unclaimed_lazy_map.descendent_lazy_maps {
            self.descendent_lazy_maps.insert(mid, lazy_map);
        }
        for (vid, vault) in unclaimed_lazy_map.descendent_vaults {
            self.descendent_vaults.insert(vid, vault);
        }
    }
}

/// Component type objects which will eventually move into a component
pub struct ComponentObjectsSet {
    /// Lazy maps which haven't been assigned to a component or lazy map yet.
    /// Keeps track of vault and lazy map descendents.
    pub lazy_maps: HashMap<Mid, UnclaimedLazyMap>,
    /// Vaults which haven't been assigned to a component or lazy map yet.
    pub vaults: HashMap<Vid, Vault>,
}

impl ComponentObjectsSet {
    pub fn new() -> Self {
        ComponentObjectsSet {
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
        }
    }

    pub fn take(
        &mut self,
        vids: Vec<Vid>,
        mids: Vec<Mid>,
    ) -> Result<ComponentObjectsSet, RuntimeError> {
        let mut vaults = HashMap::new();
        let mut lazy_maps = HashMap::new();

        for vid in vids {
            let vault = self
                .vaults
                .remove(&vid)
                .ok_or(RuntimeError::VaultNotFound(vid))?;
            vaults.insert(vid, vault);
        }

        for mid in mids {
            let lazy_map = self
                .lazy_maps
                .remove(&mid)
                .ok_or(RuntimeError::LazyMapNotFound(mid))?;
            lazy_maps.insert(mid, lazy_map);
        }

        Ok(ComponentObjectsSet { vaults, lazy_maps })
    }

    pub fn get_unclaimed_map(&mut self, mid: &Mid) -> &mut UnclaimedLazyMap {
        self.lazy_maps.get_mut(mid).unwrap()
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
