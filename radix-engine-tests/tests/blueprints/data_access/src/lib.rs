use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;
use scrypto::radix_engine_interface::api::ClientSubstateApi;

#[blueprint]
mod data_access {
    struct DataAccess {}

    impl DataAccess {
        pub fn create_component_and_read_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Component(ComponentOffset::State0),
                    false,
                )
                .unwrap();
            ScryptoEnv.sys_read_substate(lock_handle).unwrap();
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::Global(GlobalAddress::Component(component_address)),
                    SubstateOffset::Component(ComponentOffset::State0),
                    true,
                )
                .unwrap();
            ScryptoEnv
                .sys_write_substate(lock_handle, scrypto_encode(&()).unwrap())
                .unwrap();
        }

        pub fn create_component_and_read_info() {
            let component_address = Self {}.instantiate().globalize();
            ScryptoEnv
                .get_global_component_type_info(component_address)
                .unwrap();
        }
    }
}
