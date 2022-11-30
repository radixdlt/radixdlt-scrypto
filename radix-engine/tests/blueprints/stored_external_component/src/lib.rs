use scrypto::prelude::*;

blueprint! {
    struct ExternalComponent {
        external_component: Option<ComponentAddress>,
    }

    impl ExternalComponent {
        pub fn create() -> ComponentAddress {
            let external_component = Self {
                external_component: Option::None,
            }
            .instantiate()
            .globalize_no_owner();

            Self {
                external_component: Option::Some(external_component),
            }
            .instantiate()
            .globalize_no_owner()
        }

        pub fn create_and_call() -> ComponentAddress {
            let external_component = Self {
                external_component: Option::None,
            }
            .instantiate()
            .globalize_no_owner();

            let component = Self {
                external_component: Option::Some(external_component),
            }
            .instantiate();
            component.func();

            component.globalize_no_owner()
        }

        pub fn func(&mut self) {
            if let Some(component) = self.external_component {
                let component: &BorrowedGlobalComponent = borrow_component!(component);
                component.call::<()>("func", args![]);
            }
        }
    }
}
