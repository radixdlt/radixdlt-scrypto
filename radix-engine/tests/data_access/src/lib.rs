use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = call_engine(RadixEngineInput::LockSubstate(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::State),
                false,
            ));
            call_engine(RadixEngineInput::Read(lock_handle))
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = call_engine(RadixEngineInput::LockSubstate(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::State),
                true,
            ));
            call_engine(RadixEngineInput::Write(lock_handle, scrypto_encode(&())))
        }

        pub fn create_component_and_read_info() -> ComponentInfoSubstate {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = call_engine(RadixEngineInput::LockSubstate(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::Info),
                false,
            ));
            let input = RadixEngineInput::Read(lock_handle);
            call_engine(input)
        }

        pub fn create_component_and_write_info() -> () {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = call_engine(RadixEngineInput::LockSubstate(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::Info),
                true,
            ));
            let input = RadixEngineInput::Write(lock_handle, scrypto_encode(&()));
            call_engine(input)
        }
    }
}
