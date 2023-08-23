use super::StateTreeVisitor;
use radix_engine_interface::blueprints::resource::LiquidNonFungibleVault;
use radix_engine_interface::{
    blueprints::resource::LiquidFungibleResource,
    types::{NodeId, ResourceAddress},
};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub struct VaultFinder {
    vaults: IndexMap<ResourceAddress, Vec<NodeId>>,
}

impl VaultFinder {
    pub fn new() -> Self {
        VaultFinder {
            vaults: index_map_new(),
        }
    }

    pub fn to_vaults(self) -> IndexMap<ResourceAddress, Vec<NodeId>> {
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
        self.vaults
            .entry(*address)
            .or_insert_with(|| Vec::new())
            .push(vault_id);
    }

    fn visit_non_fungible_vault(
        &mut self,
        vault_id: NodeId,
        address: &ResourceAddress,
        _resource: &LiquidNonFungibleVault,
    ) {
        self.vaults
            .entry(*address)
            .or_insert_with(|| Vec::new())
            .push(vault_id);
    }
}
