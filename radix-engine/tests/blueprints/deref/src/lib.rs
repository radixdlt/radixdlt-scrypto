use radix_engine_interface::api::types::*;
use radix_engine_interface::api::wasm_input::*;
use radix_engine_interface::api::api::EngineApi;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct Deref {}

    impl Deref {
        pub fn verify_no_visible_component_nodes_on_deref(component_address: ComponentAddress) {
            {
                let mut syscalls = ScryptoEnv;
                let lock_handle = syscalls
                    .sys_lock_substate(RENodeId::Global(GlobalAddress::Component(component_address)), SubstateOffset::Component(ComponentOffset::Info), false)
                    .unwrap();
                syscalls.sys_drop_lock(lock_handle).unwrap();
            }

            let visible_node_ids: Vec<RENodeId> =
                call_engine(RadixEngineInput::GetVisibleNodeIds());

            for node_id in visible_node_ids {
                if let RENodeId::Component(..) = node_id {
                    panic!("Component Node Found: {:?}", node_id);
                }
            }
        }
    }
}
