use scrypto::engine::{api::*, call_engine};
use scrypto::prelude::*;

blueprint! {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let address = DataAddress::Component(component_address);
            let input = RadixEngineInput::ReadData(address);
            call_engine(input)
        }

        pub fn create_component_and_read_info() -> (PackageAddress, String) {
            let component_address = Self {}.instantiate().globalize();
            let address = DataAddress::ComponentInfo(component_address);
            let input = RadixEngineInput::ReadData(address);
            call_engine(input)
        }

        pub fn create_component_and_write_info() -> () {
            let component_address = Self {}.instantiate().globalize();
            let address = DataAddress::ComponentInfo(component_address);
            let input = RadixEngineInput::WriteData(address, scrypto_encode(&()));
            call_engine(input)
        }
    }
}
