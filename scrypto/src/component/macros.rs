use crate::component::*;
use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_interface::api::*;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::scrypto_encode;
use sbor::rust::prelude::*;

#[macro_export]
macro_rules! borrow_package {
    ($address:expr) => {
        $crate::component::BorrowedPackage($address.clone())
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
        .new_object(blueprint_name, vec![scrypto_encode(&state).unwrap()])
        .unwrap();
    OwnedComponent(Own(node_id))
}
