use scrypto::engine::scrypto_env::*;
use scrypto::engine_lib::engine::scrypto_env::*;
use scrypto::engine_lib::engine::types::*;
use scrypto::prelude::*;

// TODO: de-dup
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    Resource(ResourceManagerId),
    Package(PackageId),
    System(EpochManagerId),
}

blueprint! {
    struct Read {}

    impl Read {
        pub fn read_global_substate(component_address: ComponentAddress) {
            let input = RadixEngineInput::LockSubstate(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Global(GlobalOffset::Global),
                false,
            );
            let handle: LockHandle = call_engine(input);
            let input = RadixEngineInput::Read(handle);
            let _: GlobalAddressSubstate = call_engine(input);
        }
    }
}

blueprint! {
    struct NodeCreate {}

    impl NodeCreate {
        pub fn create_node_with_invalid_blueprint() {
            let input = RadixEngineInput::CreateNode(ScryptoRENode::Component(
                Runtime::package_address(),
                "invalid_blueprint".to_owned(),
                scrypto_encode(&NodeCreate {}),
            ));
            let _: ComponentId = call_engine(input);
        }

        pub fn create_node_with_invalid_package() {
            let package_address = PackageAddress::Normal([0u8; 26]);
            let input = RadixEngineInput::CreateNode(ScryptoRENode::Component(
                package_address,
                "NodeCreate".to_owned(),
                scrypto_encode(&NodeCreate {}),
            ));
            let _: ComponentId = call_engine(input);
        }
    }
}
