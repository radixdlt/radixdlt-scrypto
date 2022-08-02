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
