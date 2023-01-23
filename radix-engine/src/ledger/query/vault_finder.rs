use crate::blueprints::resources::VaultSubstate;
use crate::ledger::StateTreeVisitor;
use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::model::ResourceAddress;
use sbor::rust::vec::Vec;

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
