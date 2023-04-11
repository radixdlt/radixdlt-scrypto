use super::StateTreeVisitor;
use radix_engine_interface::{
    blueprints::resource::{LiquidFungibleResource, LiquidNonFungibleResource},
    types::{NodeId, ResourceAddress},
};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub struct VaultFinder {
    vaults: Vec<NodeId>,
    resource_address: ResourceAddress,
}

impl VaultFinder {
    pub fn new(resource_address: ResourceAddress) -> Self {
        VaultFinder {
            vaults: Vec::new(),
            resource_address,
        }
    }

    pub fn to_vaults(self) -> Vec<NodeId> {
        self.vaults
    }
}

impl StateTreeVisitor for VaultFinder {
    fn visit_fungible_vault(
        &mut self,
        vault_id: NodeId,
        address: &ResourceAddress,
        _resource: &LiquidFungibleResource,
    ) {
        if self.resource_address.eq(address) {
            self.vaults.push(vault_id);
        }
    }

    fn visit_non_fungible_vault(
        &mut self,
        vault_id: NodeId,
        address: &ResourceAddress,
        _resource: &LiquidNonFungibleResource,
    ) {
        if self.resource_address.eq(&address) {
            self.vaults.push(vault_id);
        }
    }
}
