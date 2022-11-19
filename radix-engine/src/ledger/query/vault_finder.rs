use crate::ledger::{
    QueryableSubstateStore, ReadableSubstateStore, StateTreeTraverser, StateTreeTraverserError,
    StateTreeVisitor,
};
use crate::model::VaultSubstate;
use crate::types::hash_map::Entry;
use crate::types::HashMap;
use radix_engine_interface::api::types::{RENodeId, SubstateId, VaultId};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::ResourceAddress;

pub struct VaultFinder {
    vaults: Vec<VaultId>,
    resource_address: ResourceAddress,
}

impl VaultFinder {
    pub fn new(resource_address: ResourceAddress) -> Self {
        VaultFinder {
            vaults: Vec::new(),
            resource_address,
        }
    }

    pub fn to_vaults(self) -> Vec<VaultId> {
        self.vaults
    }
}

impl StateTreeVisitor for VaultFinder {
    fn visit_vault(&mut self, vault_id: VaultId, vault: &VaultSubstate) {
        if self.resource_address.eq(&vault.0.resource_address()) {
            self.vaults.push(vault_id);
        }
    }
}
