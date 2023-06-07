use native_sdk::resource::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct TwoResourcePoolSubstate {
    /// The vaults of the resources of the pool - the maximum number of entires that this map can
    /// have is two, a single vault for each resource. This is a map as it makes the interactions
    /// simpler.
    pub vaults: [(ResourceAddress, Vault); 2],

    /// The resource manager of the pool unit resource that the pool works with.
    pub pool_unit_resource_manager: ResourceManager,
}

impl TwoResourcePoolSubstate {
    pub fn vault(&self, resource_address: ResourceAddress) -> Option<Vault> {
        self.vaults
            .iter()
            .find(|(vault_resource_address, _)| resource_address == *vault_resource_address)
            .map(|(_, vault)| Vault(vault.0.clone()))
    }
}

impl Clone for TwoResourcePoolSubstate {
    fn clone(&self) -> Self {
        let (resource_address1, vault1) = self.vaults.get(0).unwrap();
        let (resource_address2, vault2) = self.vaults.get(1).unwrap();

        Self {
            vaults: [
                (*resource_address1, Vault(vault1.0.clone())),
                (*resource_address2, Vault(vault2.0.clone())),
            ],
            pool_unit_resource_manager: self.pool_unit_resource_manager.clone(),
        }
    }
}
