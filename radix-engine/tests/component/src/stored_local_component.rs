use scrypto::prelude::*;

blueprint! {
    struct StoredLocalComponent {
        component: Component,
    }

    impl StoredLocalComponent {
        pub fn get_secret(&self) -> u32 {
            self.component.call("get_secret", vec![])
        }

        pub fn set_secret(&mut self, next: u32) {
            self.component.call("set_secret", vec![scrypto_encode(&99999u32)])
        }

        pub fn call_read_on_stored_component_in_owned_component() -> ComponentAddress {
            let package_address = Runtime::package_address();
            let component = Runtime::call_function(package_address, "LocalComponent", "new", vec![scrypto_encode(&12345u32)]);

            let my_component = Self {
                component
            }.instantiate();

            let rtn: u32 = my_component.call("get_secret", vec![]);
            assert_eq!(12345, rtn);

            my_component.globalize()
        }
    }
}
