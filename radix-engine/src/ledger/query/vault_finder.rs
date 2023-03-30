use crate::blueprints::resource::FungibleVaultInfoSubstate;
use crate::types::*;
use crate::{blueprints::resource::NonFungibleVaultInfoSubstate, ledger::StateTreeVisitor};
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::vec::Vec;

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
        info: &FungibleVaultInfoSubstate,
        _resource: &LiquidFungibleResource,
    ) {
        if self.resource_address.eq(&info.resource_address) {
            self.vaults.push(vault_id);
        }
    }

    fn visit_non_fungible_vault(
        &mut self,
        vault_id: ObjectId,
        info: &NonFungibleVaultInfoSubstate,
        _resource: &LiquidNonFungibleResource,
    ) {
        if self.resource_address.eq(&info.resource_address) {
            self.vaults.push(vault_id);
        }
    }
}
