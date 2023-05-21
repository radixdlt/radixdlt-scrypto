use scrypto::prelude::*;

#[blueprint]
mod factory {
    struct Factory {
        my_component: Option<Global<Factory>>,
    }

    impl Factory {
        pub fn create_raw() -> Global<Factory> {
            Self { my_component: None }
                .instantiate()
                .prepare_to_globalize()
                .define_roles(roles! {
                    "auth" => rule!(require(Runtime::package_token())), rule!(deny_all)
                })
                .protect_methods(protect!(
                    Method::set_address => vec!["auth"],
                ))
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
