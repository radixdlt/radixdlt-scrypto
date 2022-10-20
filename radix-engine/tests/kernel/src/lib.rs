use scrypto::engine::{api::*, types::*, utils::*};
use scrypto::prelude::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    SystemComponent(scrypto::component::Component),
    Resource(ResourceAddress),
    Package(PackageAddress),
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
            let input = RadixEngineInput::RENodeCreate(ScryptoRENode::Component(
                Runtime::package_address(),
                "invalid_blueprint".to_owned(),
                scrypto_encode(&NodeCreate {}),
            ));
            let component_id: ComponentId = call_engine(input);

            let input = RadixEngineInput::RENodeGlobalize(RENodeId::Component(component_id));
            let _: () = call_engine(input);
        }

        pub fn create_node_with_invalid_package() {
            let package_address = PackageAddress::Normal([0u8; 26]);
            let input = RadixEngineInput::RENodeCreate(ScryptoRENode::Component(
                package_address,
                "NodeCreate".to_owned(),
                scrypto_encode(&NodeCreate {}),
            ));
            let component_id: ComponentId = call_engine(input);

            let input = RadixEngineInput::RENodeGlobalize(RENodeId::Component(component_id));
            let _: () = call_engine(input);
        }
    }
}
