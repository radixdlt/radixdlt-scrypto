use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(RENodeId::Component(component_address), SubstateOffset::Component(ComponentOffset::State));
            let input = RadixEngineInput::SubstateRead(substate_id);
            call_engine(input)
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(RENodeId::Component(component_address), SubstateOffset::Component(ComponentOffset::State));
            let input = RadixEngineInput::SubstateWrite(substate_id, scrypto_encode(&()));
            call_engine(input)
        }

        pub fn create_component_and_read_info() -> ComponentInfoSubstate {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(RENodeId::Component(component_address), SubstateOffset::Component(ComponentOffset::Info));
            let input = RadixEngineInput::SubstateRead(substate_id);
            call_engine(input)
        }

        pub fn create_component_and_write_info() -> () {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(RENodeId::Component(component_address), SubstateOffset::Component(ComponentOffset::Info));
            let input = RadixEngineInput::SubstateWrite(substate_id, scrypto_encode(&()));
            call_engine(input)
        }
    }
}
