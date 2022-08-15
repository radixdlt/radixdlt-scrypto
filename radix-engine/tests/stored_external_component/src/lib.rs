use scrypto::prelude::*;

blueprint! {
    struct ExternalComponent {
        external_component: Option<ComponentAddress>,
    }

    impl ExternalComponent {
        pub fn create() -> ComponentAddress {
            let external_component = Self {
                external_component: Option::None
            }.instantiate().globalize();

            Self {
                external_component: Option::Some(external_component)
            }
            .instantiate()
            .globalize()
        }

        pub fn func(&mut self) {
            if let Some(component) = self.external_component {
                let component: &Component = borrow_component!(component);
                component.call::<()>("func", args![]);
            }
        }
    }
}
