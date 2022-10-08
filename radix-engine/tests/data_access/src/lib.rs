use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::State),
            );

            let lock_handle: LockHandle =
                call_engine(RadixEngineInput::CreateRef(substate_id.clone(), false));
            call_engine(RadixEngineInput::SubstateRead(lock_handle))
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::State),
            );
            let lock_handle: LockHandle =
                call_engine(RadixEngineInput::CreateRef(substate_id.clone(), true));
            call_engine(RadixEngineInput::SubstateWrite(
                lock_handle,
                scrypto_encode(&()),
            ))
        }

        pub fn create_component_and_read_info() -> ComponentInfoSubstate {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::Info),
            );
            let lock_handle: LockHandle =
                call_engine(RadixEngineInput::CreateRef(substate_id.clone(), false));
            let input = RadixEngineInput::SubstateRead(lock_handle);
            call_engine(input)
        }

        pub fn create_component_and_write_info() -> () {
            let component_address = Self {}.instantiate().globalize();
            let substate_id = SubstateId(
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Component(ComponentOffset::Info),
            );
            let lock_handle: LockHandle =
                call_engine(RadixEngineInput::CreateRef(substate_id.clone(), true));
            let input = RadixEngineInput::SubstateWrite(lock_handle, scrypto_encode(&()));
            call_engine(input)
        }
    }
}
