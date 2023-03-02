use crate::component::*;
use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_interface::api::ClientComponentApi;
use radix_engine_interface::data::scrypto::scrypto_encode;
use sbor::rust::collections::*;

#[macro_export]
macro_rules! borrow_package {
    ($address:expr) => {
        $crate::component::Package($address.clone())
    };
}

#[macro_export]
macro_rules! borrow_component {
    ($address:expr) => {
        $crate::component::GlobalComponentRef($address.clone())
    };
}

/// Instantiates a component.
pub fn create_component<T: ComponentState<C>, C: Component + LocalComponent>(
    blueprint_name: &str,
    state: T,
) -> OwnedComponent {
    let mut env = ScryptoEnv;
    let node_id = env
        .new_component(
            blueprint_name,
            btreemap!(
                0 => scrypto_encode(&state).unwrap()
            ),
        )
        .unwrap();
    OwnedComponent(node_id.into())
}
