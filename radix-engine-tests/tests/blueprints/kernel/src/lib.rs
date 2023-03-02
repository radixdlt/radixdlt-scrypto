use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

// TODO: de-dup
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum GlobalSubstate {
    Component(ComponentId),
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
                    RENodeId::GlobalComponent(component_address),
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                    LockFlags::read_only(),
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
                )
                .unwrap();
        }
    }
}
