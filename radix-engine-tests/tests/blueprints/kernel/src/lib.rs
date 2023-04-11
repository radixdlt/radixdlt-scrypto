use scrypto::api::substate_api::LockFlags;
use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod read {
    struct Read {}

    impl Read {
        pub fn read_global_substate(component_address: ComponentAddress) {
            ScryptoEnv
                .sys_lock_substate(
                    component_address.as_node_id(),
                    &TypeInfoOffset::TypeInfo.into(),
                    LockFlags::read_only(),
                )
                .unwrap();
        }
    }
}

#[blueprint]
mod node_create {
    struct NodeCreate {}

    impl NodeCreate {
        pub fn create_node_with_invalid_blueprint() {
            ScryptoEnv
                .new_object(
                    "invalid_blueprint",
                    vec![scrypto_encode(&NodeCreate {}).unwrap()],
                )
                .unwrap();
        }
    }
}
