use radix_engine_interface::api::types::*;
use radix_engine_interface::api::EngineApi;
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
                .sys_create_node(ScryptoRENode::Component(
                    Runtime::package_address(),
                    "invalid_blueprint".to_owned(),
                    scrypto_encode(&NodeCreate {}).unwrap(),
                ))
                .unwrap();
        }

        pub fn create_node_with_invalid_package() {
            let package_address = PackageAddress::Normal([0u8; 26]);
            ScryptoEnv
                .sys_create_node(ScryptoRENode::Component(
                    package_address,
                    "NodeCreate".to_owned(),
                    scrypto_encode(&NodeCreate {}).unwrap(),
                ))
                .unwrap();
        }
    }
}
