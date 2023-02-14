use radix_engine_interface::api::types::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;
use scrypto::radix_engine_interface::api::EngineApi;

#[blueprint]
mod data_access {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Component(ComponentOffset::State),
                    false,
                )
                .unwrap();
            ScryptoEnv.sys_read(lock_handle).unwrap();
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Component(ComponentOffset::State),
                    true,
                )
                .unwrap();
            ScryptoEnv
                .sys_write(lock_handle, scrypto_encode(&()).unwrap())
                .unwrap();
        }

        pub fn create_component_and_read_info() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Component(ComponentOffset::Info),
                    false,
                )
                .unwrap();
            ScryptoEnv.sys_read(lock_handle).unwrap();
        }

        pub fn create_component_and_write_info() -> () {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Component(ComponentOffset::Info),
                    true,
                )
                .unwrap();
            ScryptoEnv
                .sys_write(lock_handle, scrypto_encode(&()).unwrap())
                .unwrap();
        }
    }
}
