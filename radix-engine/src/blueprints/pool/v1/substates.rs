use crate::blueprints::models::*;
use crate::internal_prelude::*;
use radix_engine_interface::prelude::*;
use radix_native_sdk::resource::*;

pub mod one_resource_pool {
    use super::*;

    declare_native_blueprint_state! {
        blueprint_ident: OneResourcePool,
        blueprint_snake_case: one_resource_pool,
        features: {
        },
        fields: {
            state:  {
                ident: State,
                field_type: {
                    kind: StaticSingleVersioned,
                },
                condition: Condition::Always,
            }
        },
        collections: {
        }
    }

    pub type OneResourcePoolStateV1 = Substate;

    #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
    #[sbor(type_name = "OneResourcePoolSubstate")]
    pub struct Substate {
        /// The vault of the resources of the pool.
        pub vault: Vault,

        /// The resource manager of the pool unit resource that the pool works with.
        pub pool_unit_resource_manager: ResourceManager,
    }
}

pub mod two_resource_pool {
    use super::*;

    declare_native_blueprint_state! {
        blueprint_ident: TwoResourcePool,
        blueprint_snake_case: two_resource_pool,
        features: {
        },
        fields: {
            state:  {
                ident: State,
                field_type: {
                    kind: StaticSingleVersioned,
                },
                condition: Condition::Always,
            }
        },
        collections: {
        }
    }

    pub type TwoResourcePoolStateV1 = Substate;

    #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
    #[sbor(type_name = "TwoResourcePoolSubstate")]
    pub struct Substate {
        /// The vaults of the resources of the pool - the maximum number of entries that this map can
        /// have is two, a single vault for each resource. This is a map as it makes the interactions
        /// simpler.
        pub vaults: [(ResourceAddress, Vault); 2],

        /// The resource manager of the pool unit resource that the pool works with.
        pub pool_unit_resource_manager: ResourceManager,
    }

    impl Substate {
        pub fn vault(&self, resource_address: ResourceAddress) -> Option<Vault> {
            self.vaults
                .iter()
                .find(|(vault_resource_address, _)| resource_address == *vault_resource_address)
                .map(|(_, vault)| Vault(vault.0))
        }
    }
}

pub mod multi_resource_pool {
    use super::*;

    declare_native_blueprint_state! {
        blueprint_ident: MultiResourcePool,
        blueprint_snake_case: multi_resource_pool,
        features: {
        },
        fields: {
            state:  {
                ident: State,
                field_type: {
                    kind: StaticSingleVersioned,
                },
                condition: Condition::Always,
            }
        },
        collections: {
        }
    }

    pub type MultiResourcePoolStateV1 = Substate;

    #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
    #[sbor(type_name = "MultiResourcePoolSubstate")]
    pub struct Substate {
        /// The vaults being stored as a `IndexMap` and not as a `KeyValueStore` is intentional here!
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
        pub vaults: IndexMap<ResourceAddress, Vault>,

        /// The resource manager of the pool unit resource that the pool works with.
        pub pool_unit_resource_manager: ResourceManager,
    }
}
