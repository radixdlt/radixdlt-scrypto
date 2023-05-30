use scrypto::prelude::*;

#[blueprint]
mod apa {
    struct AllocatedAddressTest {
        store: Option<KeyValueStore<u32, ComponentAddress>>,
    }

    impl AllocatedAddressTest {
        pub fn create_and_return() -> ComponentAddress {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            address
        }

        pub fn create_and_call() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Runtime::call_method(address, "hi", scrypto_args!())
        }

        pub fn create_and_consume_within_frame() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Self::globalize_with_preallocated_address(address);
        }

        pub fn create_and_consume_in_another_frame() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            Runtime::call_function(
                Runtime::package_address(),
                "AllocatedAddressTest",
                "globalize_with_preallocated_address",
                scrypto_args!(address),
            )
        }

        pub fn create_and_store_in_key_value_store() {
            let (own, address) = Runtime::allocate_component_address(Runtime::blueprint_id());
            let store = KeyValueStore::new();
            store.insert(1u32, address);
            Self { store: Some(store) }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(address)
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
                .with_address(address)
                .globalize();
        }

        pub fn globalize_with_preallocated_address(address: ComponentAddress) {
            Self { store: None }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(address)
                .globalize();
        }
    }
}
