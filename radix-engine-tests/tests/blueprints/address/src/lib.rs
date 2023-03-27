use scrypto::prelude::*;

#[blueprint]
mod child_component {
    struct ChildComponent {
    }

    impl ChildComponent {
        pub fn create() -> ChildComponentComponent {
            Self {}.instantiate()
        }

        pub fn get_address(&self) -> ComponentAddress {
            let address = Runtime::get_global_address();
            address.into()
        }
    }
}


#[blueprint]
mod my_component {
    use child_component::*;

    struct MyComponent {
        child: ChildComponentComponent,
    }

    impl MyComponent {
        pub fn create() -> ComponentAddress {
            let child = ChildComponent::create();
            Self {
                child,
            }.instantiate().globalize()
        }

        pub fn get_address_in_local() -> ComponentAddress {
            let child = ChildComponent::create();
            let address = child.get_address();
            Self {
                child,
            }.instantiate().globalize();
            address.into()
        }

        pub fn get_address_in_parent(&self) -> ComponentAddress {
            let address = Runtime::get_global_address();
            address.into()
        }

        pub fn get_address_in_child(&self) -> ComponentAddress {
            let address = self.child.get_address();
            address.into()
        }
    }
}
