use scrypto::prelude::*;

#[blueprint]
mod external_component {
    struct ExternalComponent {
        external_component: Option<ComponentAddress>,
    }

    impl ExternalComponent {
        pub fn create() -> ComponentAddress {
            let external_component = Self {
                external_component: Option::None,
            }
            .instantiate()
            .globalize();

            Self {
                external_component: Option::Some(external_component),
            }
            .instantiate()
            .globalize()
        }

        pub fn create_and_call() -> ComponentAddress {
            let external_component = Self {
                external_component: Option::None,
            }
            .instantiate()
            .globalize();

            let component = Self {
                external_component: Option::Some(external_component),
            }
            .instantiate();
            component.func();

            component.globalize()
        }

        pub fn func(&mut self) {
            if let Some(component) = self.external_component {
                let component_ref: ExternalComponentGlobalComponentRef = component.into();
                component_ref.func();
            }
        }
    }
}
