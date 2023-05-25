use native_sdk::resource::*;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct SingleResourcePoolSubstate {
    /// The vault of the resources of the pool.
    pub vault: Own,

    /// The address of the pool unit resource that the pool works with.
    pub pool_unit_resource: ResourceAddress,
}

impl SingleResourcePoolSubstate {
    pub fn vault(&self) -> Vault {
        Vault(self.vault)
    }

    pub fn pool_unit_resource_manager(&self) -> ResourceManager {
        ResourceManager(self.pool_unit_resource)
    }
}
