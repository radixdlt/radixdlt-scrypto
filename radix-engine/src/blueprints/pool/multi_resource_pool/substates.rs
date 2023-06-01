use native_sdk::resource::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct MultiResourcePoolSubstate {
    /// The vaults being stored as a `BTreeMap` and not as a `KeyValueStore` is intentional here!
    ///
    /// All of the operations on the many pool blueprint require loading all of the vaults and doing
    /// some arithmetic, reading their state (with exception to protected deposit and withdraw).
    /// Storing this as a `KeyValueStore` only to later read the entire KVStore is pointless.
    ///
    /// Also, while protected deposits and withdraws _technically_ do not need to read the entire
    /// map of vaults they realistically do since the caller needs to know the balances to do the
    /// arithmetic they need prior to doing a deposit or withdraw. Thus, these two methods are in
    /// a realistic setting need to read that state.
    ///
    /// Additionally, size should not be a serious concern for any realistic application. The vaults
    /// map in a pool of 10 resources is just 605 bytes with 20 resources its 1205 bytes which is
    /// still reasonable. Note that most applications that would use a pool of this kind might be
    /// balancer-esc applications where the maximum number of tokens a pool can hold is 8; thus
    /// there is no concern that this map would become too big.
    ///
    /// Finally, when using this resource pool as part of a dApp all that the dApp would store is a
    /// reference to the pool. In other words, if the dApp has a method that does not interact with
    /// the pool, it is not in any way affected by how the pool stores the vaults; cost units and
    /// fees do not come into the picture there.
    pub vaults: BTreeMap<ResourceAddress, Vault>,

    /// The resource manager of the pool unit resource that the pool works with.
    pub pool_unit_resource_manager: ResourceManager,
}

impl Clone for MultiResourcePoolSubstate {
    fn clone(&self) -> Self {
        let vaults = self
            .vaults
            .iter()
            .map(|(resource_address, vault)| (resource_address.clone(), Vault(vault.0.clone())))
            .collect();
        Self {
            vaults,
            pool_unit_resource_manager: self.pool_unit_resource_manager.clone(),
        }
    }
}
