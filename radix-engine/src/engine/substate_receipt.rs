use indexmap::IndexMap;
use scrypto::engine::types::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;


pub struct CommitReceipt {
    pub down_substates: HashSet<(Hash, u32)>,
    pub up_substates: Vec<(Hash, u32)>,
}

impl CommitReceipt {
    fn new() -> Self {
        CommitReceipt {
            down_substates: HashSet::new(),
            up_substates: Vec::new(),
        }
    }

    fn down(&mut self, id: (Hash, u32)) {
        self.down_substates.insert(id);
    }

    fn up(&mut self, id: (Hash, u32)) {
        self.up_substates.push(id);
    }
}

pub struct SubstateReceipt {
    pub packages: IndexMap<PackageAddress, SubstateUpdate<Package>>,
    pub components: IndexMap<ComponentAddress, SubstateUpdate<Component>>,
    pub resource_managers: IndexMap<ResourceAddress, SubstateUpdate<ResourceManager>>,
    pub vaults: HashMap<(ComponentAddress, VaultId), SubstateUpdate<Vault>>,
    pub non_fungibles: HashMap<NonFungibleAddress, SubstateUpdate<Option<NonFungible>>>,
    pub lazy_map_entries: HashMap<(ComponentAddress, LazyMapId, Vec<u8>), SubstateUpdate<Vec<u8>>>,
}

impl SubstateReceipt {
    /// Returns new packages created so far.
    pub fn new_package_addresses(&self) -> Vec<PackageAddress> {
        let mut package_addresses = Vec::new();
        for (package_address, update) in self.packages.iter() {
            if let None = update.prev_id {
                package_addresses.push(package_address.clone());
            }
        }
        package_addresses
    }

    /// Returns new components created so far.
    pub fn new_component_addresses(&self) -> Vec<ComponentAddress> {
        let mut component_addresses = Vec::new();
        for (component_address, update) in self.components.iter() {
            if let None = update.prev_id {
                component_addresses.push(component_address.clone());
            }
        }
        component_addresses
    }

    /// Returns new resource addresses created so far.
    pub fn new_resource_addresses(&self) -> Vec<ResourceAddress> {
        let mut resource_addresses = Vec::new();
        for (resource_address, update) in self.resource_managers.iter() {
            if let None = update.prev_id {
                resource_addresses.push(resource_address.clone());
            }
        }
        resource_addresses
    }

    /// Commits changes to the underlying ledger.
    /// Currently none of these objects are deleted so all commits are puts
    pub fn commit<S: WriteableSubstateStore>(mut self, hash: Hash, store: &mut S) -> CommitReceipt {
        let mut receipt = CommitReceipt::new();
        let mut id_gen = SubstateIdGenerator::new(hash);

        let package_addresses: Vec<PackageAddress> = self.packages.keys().cloned().collect();
        for package_address in package_addresses {
            let package = self.packages.remove(&package_address).unwrap();

            if let Some(prev_id) = package.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            store.put_encoded_substate(&package_address, &package.value, phys_id);
        }

        let component_addresses: Vec<ComponentAddress> = self.components.keys().cloned().collect();
        for component_address in component_addresses {
            let component = self.components.remove(&component_address).unwrap();

            if let Some(prev_id) = component.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            store.put_encoded_substate(&component_address, &component.value, phys_id);
        }

        let resource_addresses: Vec<ResourceAddress> =
            self.resource_managers.keys().cloned().collect();
        for resource_address in resource_addresses {
            let resource_manager = self.resource_managers.remove(&resource_address).unwrap();

            if let Some(prev_id) = resource_manager.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            store.put_encoded_substate(
                &resource_address,
                &resource_manager.value,
                phys_id,
            );
        }

        let entry_ids: Vec<(ComponentAddress, LazyMapId, Vec<u8>)> =
            self.lazy_map_entries.keys().cloned().collect();
        for entry_id in entry_ids {
            let entry = self.lazy_map_entries.remove(&entry_id).unwrap();
            if let Some(prev_id) = entry.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            let (component_address, lazy_map_id, key) = entry_id;
            store.put_encoded_grand_child_substate(
                &component_address,
                &lazy_map_id,
                &key,
                &entry.value,
                phys_id,
            );
        }

        let vault_ids: Vec<(ComponentAddress, VaultId)> = self.vaults.keys().cloned().collect();
        for vault_id in vault_ids {
            let vault = self.vaults.remove(&vault_id).unwrap();
            if let Some(prev_id) = vault.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            let (component_address, vault_id) = vault_id;
            store.put_encoded_child_substate(
                &component_address,
                &vault_id,
                &vault.value,
                phys_id,
            );
        }

        let non_fungible_addresses: Vec<NonFungibleAddress> =
            self.non_fungibles.keys().cloned().collect();
        for non_fungible_address in non_fungible_addresses {
            let non_fungible = self.non_fungibles.remove(&non_fungible_address).unwrap();
            if let Some(prev_id) = non_fungible.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            store.put_encoded_child_substate(
                &non_fungible_address.resource_address(),
                &non_fungible_address.non_fungible_id(),
                &non_fungible.value,
                phys_id,
            );
        }

        receipt
    }
}
