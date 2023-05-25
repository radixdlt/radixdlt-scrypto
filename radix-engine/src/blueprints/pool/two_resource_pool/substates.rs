use native_sdk::resource::*;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct TwoResourcePoolSubstate {
    /// The vaults of the resources of the pool - the maximum number of entires that this map can
    /// have is two, a single vault for each resource. This is a map as it makes the interactions
    /// simpler.
    pub vaults: [(ResourceAddress, Own); 2],

    /// The address of the pool unit resource that the pool works with.
    pub pool_unit_resource: ResourceAddress,
}

impl TwoResourcePoolSubstate {
    pub fn vaults(&self) -> [(ResourceAddress, Vault); 2] {
        self.vaults
            .iter()
            .map(|(resource_address, vault)| (*resource_address, Vault(*vault)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub fn vault(&self, resource_address: ResourceAddress) -> Option<Vault> {
        self.vaults
            .iter()
            .find(|(vault_resource_address, _)| resource_address == *vault_resource_address)
            .map(|(_, vault)| Vault(*vault))
    }

    pub fn pool_unit_resource_manager(&self) -> ResourceManager {
        ResourceManager(self.pool_unit_resource)
    }
}
