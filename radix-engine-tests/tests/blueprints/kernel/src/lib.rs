use scrypto::api::*;
use scrypto::prelude::*;

#[blueprint]
mod node_create {
    struct NodeCreate {}

    impl NodeCreate {
        pub fn create_node_with_invalid_blueprint() {
            ScryptoEnv
                .new_simple_object("invalid_blueprint", vec![FieldValue::new(&NodeCreate {})])
                .unwrap();
        }
    }
}
