use scrypto::api::substate_api::LockFlags;
use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod data_access {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    component_address.as_node_id(),
                    &ComponentOffset::State0.into(),
                    LockFlags::read_only(),
                )
                .unwrap();
            ScryptoEnv.sys_read_substate(lock_handle).unwrap();
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    component_address.as_node_id(),
                    &ComponentOffset::State0.into(),
                    LockFlags::MUTABLE,
                )
                .unwrap();
            ScryptoEnv
                .sys_write_substate(lock_handle, scrypto_encode(&()).unwrap())
                .unwrap();
        }

        pub fn create_component_and_read_info() {
            let component_address = Self {}.instantiate().globalize();
            ScryptoEnv
                .get_object_info(component_address.as_node_id())
                .unwrap();
        }
    }
}
