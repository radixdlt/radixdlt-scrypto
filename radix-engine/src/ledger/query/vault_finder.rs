use crate::{blueprints::resource::VaultInfoSubstate, ledger::StateTreeVisitor};
use radix_engine_interface::{
    api::types::{ResourceAddress, VaultId},
    blueprints::resource::{LiquidFungibleResource, LiquidNonFungibleResource},
};
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
    fn visit_fungible_vault(
        &mut self,
        vault_id: VaultId,
        info: &VaultInfoSubstate,
        _resource: &LiquidFungibleResource,
    ) {
        if self.resource_address.eq(&info.resource_address) {
            self.vaults.push(vault_id);
        }
    }

    fn visit_non_fungible_vault(
        &mut self,
        vault_id: VaultId,
        info: &VaultInfoSubstate,
        _resource: &LiquidNonFungibleResource,
    ) {
        if self.resource_address.eq(&info.resource_address) {
            self.vaults.push(vault_id);
        }
    }
}
