use scrypto::prelude::*;

blueprint! {
    struct StoredKVLocalComponent {
        components: KeyValueStore<u32, Component>,
    }

    impl StoredKVLocalComponent {
        pub fn parent_get_secret(&self) -> u32 {
            self.components
                .get(&0u32)
                .unwrap()
                .call("get_secret", vec![])
        }

        pub fn parent_set_secret(&mut self, next: u32) {
            self.components
                .get(&0u32)
                .unwrap()
                .call("set_secret", vec![scrypto_encode(&next)])
        }

        pub fn new(secret: u32) -> Component {
            let package_address = Runtime::package_address();
            let component = Runtime::call_function(
                package_address,
                "LocalComponent",
                "new",
                vec![scrypto_encode(&secret)],
            );

            let components = KeyValueStore::new();
            components.insert(0u32, component);

            Self { components }.instantiate()
        }

        pub fn new_global(secret: u32) -> ComponentAddress {
            Self::new(secret).globalize()
        }

        pub fn call_read_on_stored_component_in_owned_component() -> ComponentAddress {
            let my_component = Self::new(12345);

            let rtn: u32 = my_component.call("parent_get_secret", vec![]);
            assert_eq!(12345, rtn);

            my_component.globalize()
        }

        pub fn call_write_on_stored_component_in_owned_component() -> ComponentAddress {
            let my_component = Self::new(12345);

            let _: () = my_component.call("parent_set_secret", vec![scrypto_encode(&99999u32)]);
            let rtn: u32 = my_component.call("parent_get_secret", vec![]);
            assert_eq!(99999, rtn);

            my_component.globalize()
        }
    }
}
