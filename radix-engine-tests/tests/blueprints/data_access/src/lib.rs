use scrypto::api::substate_api::LockFlags;
use scrypto::api::ClientComponentApi;
use scrypto::api::ClientSubstateApi;
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
                    RENodeId::GlobalComponent(component_address),
                    SubstateOffset::Component(ComponentOffset::State0),
                    LockFlags::read_only(),
                )
                .unwrap();
            ScryptoEnv.sys_read_substate(lock_handle).unwrap();
        }

        pub fn create_component_and_write_state() {
            let component_address = Self {}.instantiate().globalize();
            let lock_handle: LockHandle = ScryptoEnv
                .sys_lock_substate(
                    RENodeId::GlobalComponent(component_address),
                    SubstateOffset::Component(ComponentOffset::State0),
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
                .get_component_type_info(RENodeId::GlobalComponent(component_address))
                .unwrap();
        }
    }
}
