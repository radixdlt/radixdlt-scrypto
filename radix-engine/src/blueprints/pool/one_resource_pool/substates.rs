use native_sdk::resource::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct OneResourcePoolSubstate {
    /// The vault of the resources of the pool.
    pub vault: Vault,

    /// The resource manager of the pool unit resource that the pool works with.
    pub pool_unit_resource_manager: ResourceManager,
}

impl Clone for OneResourcePoolSubstate {
    fn clone(&self) -> Self {
        Self {
            vault: Vault(self.vault.0.clone()),
            pool_unit_resource_manager: self.pool_unit_resource_manager.clone(),
        }
    }
}
