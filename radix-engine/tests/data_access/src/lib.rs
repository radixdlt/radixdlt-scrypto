use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId::ComponentState(component_address);
            let input = RadixEngineInput::SubstateRead(substate_id);
            call_engine(input)
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId::ComponentState(component_address);
            let input = RadixEngineInput::SubstateWrite(substate_id, scrypto_encode(&()));
            call_engine(input)
        }

        pub fn create_component_and_read_info() -> (PackageAddress, String) {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId::ComponentInfo(component_address, true);
            let input = RadixEngineInput::SubstateRead(substate_id);
            call_engine(input)
        }

        pub fn create_component_and_write_info() -> () {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId::ComponentInfo(component_address, true);
            let input = RadixEngineInput::SubstateWrite(substate_id, scrypto_encode(&()));
            call_engine(input)
        }
    }
}
