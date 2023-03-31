use scrypto::prelude::*;

#[blueprint]
mod child_component {
    struct ChildComponent {
        to_call: ComponentAddress,
    }

    impl ChildComponent {
        pub fn create(to_call: ComponentAddress) -> ChildComponentComponent {
            Self { to_call }.instantiate()
        }

        pub fn get_address(&self) -> ComponentAddress {
            Runtime::global_address()
        }

        pub fn call_other_component(&self) {
            let _: () = Runtime::call_method(
                self.to_call,
                "protected_method",
                scrypto_args!(Runtime::global_address()),
            );
        }
    }
}

#[blueprint]
mod my_component {
    use child_component::*;

    struct MyComponent {
        child: ChildComponentComponent,
        to_call: ComponentAddress,
    }

    impl MyComponent {
        pub fn create(to_call: ComponentAddress) -> ComponentAddress {
            let child = ChildComponent::create(to_call);
            Self { child, to_call }.instantiate().globalize()
        }

        pub fn get_address_in_parent(&self) -> ComponentAddress {
            Runtime::global_address()
        }

        pub fn get_address_in_owned(&self) -> ComponentAddress {
            self.child.get_address()
        }

        pub fn get_address_in_local(to_call: ComponentAddress) -> ComponentAddress {
            let child = ChildComponent::create(to_call);
            let address = child.get_address();
            Self { child, to_call }.instantiate().globalize();
            address
        }

        pub fn get_address_in_local_of_parent_method(
            &self,
            to_call: ComponentAddress,
        ) -> ComponentAddress {
            Self::get_address_in_local(to_call)
        }

        pub fn call_other_component_with_wrong_address(&self) {
            let address = self.to_call;
            Runtime::call_method(self.to_call, "protected_method", scrypto_args!(address))
        }

        pub fn call_other_component_in_parent(&self) {
            Runtime::call_method(
                self.to_call,
                "protected_method",
                scrypto_args!(Runtime::global_address()),
            )
        }

        pub fn call_other_component_in_child(&self) {
            self.child.call_other_component();
        }
    }
}

#[blueprint]
mod called_component {
    struct CalledComponent {}

    impl CalledComponent {
        pub fn create() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn protected_method(&self, component_address: ComponentAddress) {
            let global_id = NonFungibleGlobalId::from_component_address(&component_address);
            Runtime::assert_access_rule(rule!(require(global_id)));
        }
    }
}
