use scrypto::prelude::*;

#[blueprint]
mod external_component {
    struct ExternalComponent {
        external_component: Option<Global<ExternalComponent>>,
    }

    impl ExternalComponent {
        pub fn create() -> Global<ExternalComponent> {
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

        pub fn create_and_call() -> Global<ExternalComponent> {
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
            if let Some(component) = &self.external_component {
                component.func();
            }
        }
    }
}
