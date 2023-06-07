use scrypto::api::ClientObjectApi;
use scrypto::prelude::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

#[blueprint]
mod apa {
    struct AllocatedAddressTest {
        store: Option<KeyValueStore<u32, ComponentAddress>>,
    }

    impl AllocatedAddressTest {
        pub fn create_and_return() -> (Owned<AnyComponent>, ComponentAddress) {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            (own, address)
        }

        pub fn create_and_drop() {
            let (own, _address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            ScryptoEnv.drop_object(own.0.handle().as_node_id()).unwrap();
        }

        pub fn create_and_pass_address() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            let _: () = Runtime::call_function(
                Runtime::package_address(),
                "AllocatedAddressTest",
                "receive_address",
                scrypto_args!(address),
            );
            Self::globalize_with_preallocated_address(own);
        }

        pub fn create_and_call() {
            let (_own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Runtime::call_method(address, "hi", scrypto_args!())
        }

        pub fn create_and_consume_within_frame() {
            let (own, _address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Self::globalize_with_preallocated_address(own);
        }

        pub fn create_and_consume_in_another_frame() {
            let (own, _address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Runtime::call_function(
                Runtime::package_address(),
                "AllocatedAddressTest",
                "globalize_with_preallocated_address",
                scrypto_args!(own),
            )
        }

        pub fn create_and_store_in_key_value_store() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            let store = KeyValueStore::new();
            store.insert(1u32, address);
            Self { store: Some(store) }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize();
        }

        pub fn create_and_store_in_metadata() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Self { store: None }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .metadata(metadata!(
                    "key" => GlobalAddress::from(address),
                ))
                .with_address(own)
                .globalize();
        }

        pub fn globalize_with_preallocated_address(own: Owned<AnyComponent>) {
            Self { store: None }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize();
        }

        pub fn receive_address(address: ComponentAddress) {
            info!("Received: {:?}", address);
        }
    }
}
