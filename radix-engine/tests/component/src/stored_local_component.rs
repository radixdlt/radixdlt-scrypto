use scrypto::prelude::*;

blueprint! {
    struct StoredLocalComponent {
        component: crate::local_component::LocalComponent,
    }

    impl StoredLocalComponent {
        pub fn parent_get_secret(&self) -> u32 {
            self.component.get_secret()
        }

        pub fn parent_set_secret(&mut self, next: u32) {
            self.component.set_secret(next)
        }

        pub fn new(secret: u32) -> crate::stored_local_component::StoredLocalComponent {
            let component = crate::local_component::LocalComponent::new(secret);

            Self { component }.instantiate()
        }

        pub fn new_global(secret: u32) -> ComponentAddress {
            Self::new(secret).globalize()
        }

        pub fn call_read_on_stored_component_in_owned_component() -> ComponentAddress {
            let my_component = Self::new(12345);
            let rtn = my_component.parent_get_secret();
            assert_eq!(12345, rtn);

            my_component.globalize()
        }

        pub fn call_write_on_stored_component_in_owned_component() -> ComponentAddress {
            let my_component = Self::new(12345);

            my_component.parent_set_secret(99999);
            let rtn = my_component.parent_get_secret();
            assert_eq!(99999, rtn);

            my_component.globalize()
        }
    }
}
