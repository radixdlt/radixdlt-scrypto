use radix_engine_interface::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

// TODO: de-dup
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum GlobalAddressSubstate {
    Component(ComponentId),
    Resource(ResourceManagerId),
    Package(PackageId),
    EpochManager(EpochManagerId),
    Clock(ClockId),
}

#[blueprint]
mod read {
    struct Read {}

    impl Read {
        pub fn read_global_substate(component_address: ComponentAddress) {
            ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Global(GlobalOffset::Global),
                    false,
                )
                .unwrap();
        }
    }
}

#[blueprint]
mod node_create {
    struct NodeCreate {}

    impl NodeCreate {
        pub fn create_node_with_invalid_blueprint() {
            ScryptoEnv
                .new_component(
                    "invalid_blueprint",
                    btreemap!(
                        0 => scrypto_encode(&NodeCreate {}).unwrap()
                    ),
                    Vec::default(),
                    RoyaltyConfig::default(),
                    BTreeMap::default(),
                )
                .unwrap();
        }
    }
}
