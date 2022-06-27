use scrypto::prelude::*;

blueprint! {
    struct StoredLocalComponent {
        component: crate::local_component::component::LocalComponent,
    }

    impl StoredLocalComponent {
        pub fn parent_get_secret(&self) -> u32 {
            self.component.call("get_secret", vec![])
        }

        pub fn parent_set_secret(&mut self, next: u32) {
            self.component
                .call("set_secret", vec![scrypto_encode(&next)])
        }

        pub fn new(secret: u32) -> component::StoredLocalComponent {
            let component = crate::local_component::component::LocalComponent::new(secret);

            Self { component }.instantiate()
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
