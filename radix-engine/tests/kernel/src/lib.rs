use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct Globalize {}

    impl Globalize {
        pub fn globalize_kv_store() {
            let kv_store: KeyValueStore<String, String> = KeyValueStore::new();
            let node_id = RENodeId::KeyValueStore(kv_store.id);
            let input = RadixEngineInput::RENodeGlobalize(node_id);
            call_engine(input)
        }

        pub fn globalize_bucket() {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(Decimal::from(10));
            let node_id = RENodeId::Bucket(bucket.0);
            let input = RadixEngineInput::RENodeGlobalize(node_id);
            call_engine(input)
        }

        pub fn globalize_proof() {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(Decimal::from(10));
            let proof = bucket.create_proof();
            let node_id = RENodeId::Proof(proof.0);
            let input = RadixEngineInput::RENodeGlobalize(node_id);
            call_engine(input)
        }

        pub fn globalize_vault() {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(Decimal::from(10));
            let vault = Vault::with_bucket(bucket);
            let node_id = RENodeId::Vault(vault.0);
            let input = RadixEngineInput::RENodeGlobalize(node_id);
            call_engine(input)
        }
    }
}

blueprint! {
    struct Invoke {}

    impl Invoke {
        pub fn call_invalid_scrypto_call_on_vault() {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(Decimal::from(10));
            let vault = Vault::with_bucket(bucket);
            let node_id = RENodeId::Vault(vault.0);

            let input = RadixEngineInput::invoke_native(
                NativeFnIdent::Method(NativeMethodIdent {
                    method_ident: MethodIdent::Scrypto("test".to_string()),
                    receiver: Receiver::Ref(node_id),
                }),
                args!(),
            );
            let _: () = call_engine(input);
        }
    }
}

blueprint! {
    struct NodeCreate {}

    impl NodeCreate {
        pub fn create_node_with_invalid_blueprint() {
            let input = RadixEngineInput::RENodeCreate(ScryptoRENode::Component(
                Runtime::package_address(),
                "invalid_blueprint".to_owned(),
                scrypto_encode(&NodeCreate {}),
            ));
            let component_id: ComponentId = call_engine(input);

            let input = RadixEngineInput::RENodeGlobalize(RENodeId::Component(component_id));
            let _: () = call_engine(input);
        }

        pub fn create_node_with_invalid_package() {
            let package_address = PackageAddress::Normal([0u8; 26]);
            let input = RadixEngineInput::RENodeCreate(ScryptoRENode::Component(
                package_address,
                "NodeCreate".to_owned(),
                scrypto_encode(&NodeCreate {}),
            ));
            let component_id: ComponentId = call_engine(input);

            let input = RadixEngineInput::RENodeGlobalize(RENodeId::Component(component_id));
            let _: () = call_engine(input);
        }
    }
}
