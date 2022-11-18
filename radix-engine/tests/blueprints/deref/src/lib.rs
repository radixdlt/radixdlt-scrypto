use radix_engine_interface::api::types::*;
use radix_engine_interface::api::wasm_input::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct Deref {}

    impl Deref {
        pub fn verify_no_new_visible_nodes_on_deref(component_address: ComponentAddress) {
            let visible_node_ids: Vec<RENodeId> =
                call_engine(RadixEngineInput::GetVisibleNodeIds());
            borrow_component!(component_address).package_address();
            let next_visible_node_ids: Vec<RENodeId> =
                call_engine(RadixEngineInput::GetVisibleNodeIds());
            assert_eq!(visible_node_ids, next_visible_node_ids);
        }
    }
}
