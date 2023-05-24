use native_sdk::resource::*;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_common::math::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct TwoResourcePoolSubstate {
    /// The vault of the resources of the pool.
    pub vaults: (Own, Own),

    /// The address of the pool unit resource that the pool works with.
    pub pool_unit_resource: ResourceAddress,

    /// The amount of pool unit resources that was minted when the pool was initially created. If
    /// [`None`] then no resources have been contributed to this pool and not pool units have been
    /// minted
    pub initial_pool_unit_amount: Option<Decimal>,
}

impl TwoResourcePoolSubstate {
    pub fn vaults(&self) -> (Vault, Vault) {
        (Vault(self.vaults.0), Vault(self.vaults.1))
    }

    pub fn pool_unit_resource_manager(&self) -> ResourceManager {
        ResourceManager(self.pool_unit_resource)
    }
}
