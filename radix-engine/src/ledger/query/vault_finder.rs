use crate::ledger::StateTreeVisitor;
use crate::types::*;
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::vec::Vec;
use crate::blueprints::resource::LiquidNonFungibleVault;

pub struct VaultFinder {
    vaults: Vec<ObjectId>,
    resource_address: ResourceAddress,
}

impl VaultFinder {
    pub fn new(resource_address: ResourceAddress) -> Self {
        VaultFinder {
            vaults: Vec::new(),
            resource_address,
        }
    }

    pub fn to_vaults(self) -> Vec<ObjectId> {
        self.vaults
    }
}

impl StateTreeVisitor for VaultFinder {
    fn visit_fungible_vault(
        &mut self,
        vault_id: ObjectId,
        address: &ResourceAddress,
        _resource: &LiquidFungibleResource,
    ) {
        if self.resource_address.eq(address) {
            self.vaults.push(vault_id);
        }
    }

    fn visit_non_fungible_vault(
        &mut self,
        vault_id: ObjectId,
        address: &ResourceAddress,
        _resource: &LiquidNonFungibleVault,
    ) {
        if self.resource_address.eq(&address) {
            self.vaults.push(vault_id);
        }
    }
}
