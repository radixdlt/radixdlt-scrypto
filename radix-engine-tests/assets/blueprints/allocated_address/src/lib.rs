use scrypto::prelude::*;

#[blueprint]
mod another {
    struct AnotherBlueprint {}

    impl AnotherBlueprint {}
}

#[blueprint]
mod apa {
    struct AllocatedAddressTest {
        store: Option<KeyValueStore<u32, ComponentAddress>>,
    }

    impl AllocatedAddressTest {
        pub fn create_and_return() -> (GlobalAddressReservation, ComponentAddress) {
            let (own, address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            (own, address)
        }

        pub fn create_and_pass_address() {
            let (own, address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            Blueprint::<AllocatedAddressTest>::receive_address(address);
            Self::globalize_with_preallocated_address(own);
        }

        pub fn create_and_call() {
            let (_own, address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            ScryptoVmV1Api::object_call(address.as_node_id(), "hi", scrypto_args!());
        }

        pub fn create_and_consume_within_frame() {
            let (own, _address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            Self::globalize_with_preallocated_address(own);
        }

        pub fn create_and_consume_with_mismatching_blueprint() {
            let (own, _address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            crate::another::AnotherBlueprint {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize();
        }

        pub fn create_and_consume_in_another_frame() {
            let (own, _address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            Blueprint::<AllocatedAddressTest>::globalize_with_preallocated_address(own);
        }

        pub fn create_and_store_in_key_value_store() {
            let (own, address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            let store = KeyValueStore::new();
            store.insert(1u32, address);
            Self { store: Some(store) }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize();
        }

        pub fn create_and_store_in_metadata() {
            let (own, address) =
                Runtime::allocate_component_address(AllocatedAddressTest::blueprint_id());
            Self { store: None }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "key" => GlobalAddress::from(address), locked;
                    }
                ))
                .with_address(own)
                .globalize();
        }

        pub fn globalize_with_preallocated_address(own: GlobalAddressReservation) {
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
