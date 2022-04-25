use scrypto::rust::ops::RangeFull;
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

pub enum SubstateInstruction {
    Down(Hash, u32),
    Up(PackageAddress, Package),
}

pub struct SubstateReceipt {
    pub packages: Vec<SubstateInstruction>,
    pub components: IndexMap<ComponentAddress, SubstateUpdate<Component>>,
    pub resource_managers: IndexMap<ResourceAddress, SubstateUpdate<ResourceManager>>,
    pub vaults: IndexMap<(ComponentAddress, VaultId), SubstateUpdate<Vault>>,
    pub non_fungibles: IndexMap<NonFungibleAddress, SubstateUpdate<Option<NonFungible>>>,
    pub lazy_map_entries: IndexMap<(ComponentAddress, LazyMapId, Vec<u8>), SubstateUpdate<Vec<u8>>>,
}

impl SubstateReceipt {
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

        for instruction in self.packages.drain(RangeFull) {
            match instruction {
                SubstateInstruction::Down(hash, index) => receipt.down((hash, index)),
                SubstateInstruction::Up(package_address, package) => {
                    let phys_id = id_gen.next();
                    receipt.up(phys_id);
                    store.put_encoded_substate(&package_address, &package, phys_id);
                }
            }
        }

        for (component_address, component) in self.components.drain(RangeFull) {
            if let Some(prev_id) = component.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            store.put_encoded_substate(&component_address, &component.value, phys_id);
        }

        for (resource_address, resource_manager) in self.resource_managers.drain(RangeFull) {
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

        for (entry_id, entry) in self.lazy_map_entries.drain(RangeFull) {
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

        for (vault_id, vault) in self.vaults.drain(RangeFull) {
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

        for (non_fungible_address, non_fungible) in self.non_fungibles.drain(RangeFull) {
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
