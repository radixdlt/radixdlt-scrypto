use scrypto::prelude::*;

#[blueprint]
mod factory {
    enable_method_auth! {
        methods {
            set_address => restrict_to: [OWNER];
        }
    }

    struct Factory {
        my_component: Option<Global<Factory>>,
    }

    impl Factory {
        pub fn create_raw() -> Global<Factory> {
            Self { my_component: None }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(require(Runtime::package_token()))))
                .globalize()
        }

        pub fn create() -> Global<Factory> {
            let component = Self::create_raw();
            component.set_address(component.clone());
            component
        }

        pub fn set_address(&mut self, my_component: Global<Factory>) {
            self.my_component = Some(my_component);
        }
    }
}
