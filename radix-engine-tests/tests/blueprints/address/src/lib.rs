use scrypto::prelude::*;

#[blueprint]
mod child_component {
    use called_component::*;

    struct ChildComponent {
        to_call: Global<CalledComponentComponent>,
    }

    impl ChildComponent {
        pub fn create(to_call: Global<CalledComponentComponent>) -> ChildComponentComponent {
            Self { to_call }.instantiate().own()
        }

        pub fn get_global_address(&self) -> ComponentAddress {
            Runtime::global_address()
        }

        pub fn call_other_component(&self, child: bool) {
            self.to_call
                .protected_method(Runtime::global_address(), child);
        }

        pub fn assert_check_on_package(&self, package_address: PackageAddress) {
            Runtime::assert_access_rule(rule!(require(package_of_direct_caller(package_address))));
        }

        pub fn assert_check_on_global_blueprint_caller(&self, blueprint: Blueprint) {
            Runtime::assert_access_rule(rule!(require(global_caller(blueprint))));
        }
    }
}

#[blueprint]
mod my_component {
    use called_component::*;
    use child_component::*;

    struct MyComponent {
        child: ChildComponentComponent,
        to_call: Global<CalledComponentComponent>,
    }

    impl MyComponent {
        pub fn create(to_call: Global<CalledComponentComponent>) -> Global<MyComponentComponent> {
            let child = ChildComponent::create(to_call.clone());
            Self { child, to_call }.instantiate().globalize()
        }

        pub fn get_global_address_in_parent(&self) -> ComponentAddress {
            Runtime::global_address()
        }

        pub fn get_global_address_in_owned(&self) -> ComponentAddress {
            self.child.get_global_address()
        }

        pub fn get_global_address_in_local(
            to_call: Global<CalledComponentComponent>,
        ) -> ComponentAddress {
            let child = ChildComponent::create(to_call.clone());
            let address = child.get_global_address();
            Self { child, to_call }.instantiate().globalize();
            address
        }

        pub fn get_global_address_in_local_of_parent_method(
            &self,
            to_call: Global<CalledComponentComponent>,
        ) -> ComponentAddress {
            Self::get_global_address_in_local(to_call)
        }

        pub fn call_other_component_with_wrong_address(&self) {
            let address = self.to_call.component_address();
            self.to_call.protected_method(address, false);
        }

        pub fn call_other_component(&self, child: bool, called_child: bool) {
            if child {
                self.child.call_other_component(called_child);
            } else {
                self.to_call
                    .protected_method(Runtime::global_address(), called_child);
            }
        }

        pub fn assert_check_on_global_blueprint_caller(&self, blueprint: Blueprint, child: bool) {
            if child {
                self.child
                    .assert_check_on_global_blueprint_caller(blueprint);
            } else {
                Runtime::assert_access_rule(rule!(require(global_caller(blueprint))));
            }
        }

        pub fn assert_check_on_package(&self, package_address: PackageAddress, child: bool) {
            if child {
                self.child.assert_check_on_package(package_address);
            } else {
                Runtime::assert_access_rule(rule!(require(package_of_direct_caller(
                    package_address
                ))));
            }
        }
    }
}

#[blueprint]
mod called_component {
    use called_component_child::*;

    struct CalledComponent {
        child: CalledComponentChildComponent,
    }

    impl CalledComponent {
        pub fn create() -> Global<CalledComponentComponent> {
            let child = CalledComponentChild::create();
            Self { child }.instantiate().globalize()
        }

        pub fn protected_method(&self, component_address: ComponentAddress, child: bool) {
            if child {
                self.child.protected_method(component_address);
            } else {
                Runtime::assert_access_rule(rule!(require(global_caller(component_address))));
                assert_ne!(Runtime::global_address(), component_address.into());
            }
        }
    }
}

#[blueprint]
mod called_component_child {
    struct CalledComponentChild {}

    impl CalledComponentChild {
        pub fn create() -> CalledComponentChildComponent {
            Self {}.instantiate().own()
        }

        pub fn protected_method(&self, component_address: ComponentAddress) {
            Runtime::assert_access_rule(rule!(require(global_caller(component_address))));
            assert_ne!(Runtime::global_address(), component_address.into());
        }
    }
}

#[blueprint]
mod preallocation_component {
    struct PreallocationComponent {}

    impl PreallocationComponent {
        pub fn create_with_preallocated_address() -> Global<PreallocationComponentComponent> {
            let component_address = Runtime::preallocate_global_component_address();
            Self {}
                .instantiate()
                .attach_address(component_address)
                .globalize()
        }

        pub fn create_with_unused_preallocated_address_1() -> Global<PreallocationComponentComponent>
        {
            let component_address = Runtime::preallocate_global_component_address();
            Runtime::preallocate_global_component_address();
            Self {}
                .instantiate()
                .attach_address(component_address)
                .globalize()
        }

        pub fn create_with_unused_preallocated_address_2() -> Global<PreallocationComponentComponent>
        {
            Runtime::preallocate_global_component_address();
            Self {}.instantiate().globalize()
        }

        pub fn create_two_with_same_address() -> (
            Global<PreallocationComponentComponent>,
            Global<PreallocationComponentComponent>,
        ) {
            let component_address = Runtime::preallocate_global_component_address();
            let one = Self {}
                .instantiate()
                .attach_address(component_address)
                .globalize();
            let two = Self {}
                .instantiate()
                .attach_address(component_address)
                .globalize();
            (one, two)
        }
    }
}

#[blueprint]
mod preallocation_smuggler_component {
    struct PreallocationSmugglerComponent {
        preallocated_address: Option<GlobalAddress>,
    }

    impl PreallocationSmugglerComponent {
        pub fn create_empty() -> Global<PreallocationSmugglerComponentComponent> {
            Self {
                preallocated_address: None,
            }
            .instantiate()
            .globalize()
        }

        pub fn create_empty_at_address_bytes(
            preallocated_address_bytes: [u8; 30],
        ) -> Global<PreallocationSmugglerComponentComponent> {
            let component_address =
                unsafe { ComponentAddress::new_unchecked(preallocated_address_bytes) };
            Self {
                preallocated_address: None,
            }
            .instantiate()
            .attach_address(component_address)
            .globalize()
        }

        pub fn create_empty_at_address(
            preallocated_address: ComponentAddress,
        ) -> Global<PreallocationSmugglerComponentComponent> {
            Self {
                preallocated_address: None,
            }
            .instantiate()
            .attach_address(preallocated_address)
            .globalize()
        }

        pub fn create_with_smuggled_address() -> Global<PreallocationSmugglerComponentComponent> {
            Self {
                preallocated_address: Some(Runtime::preallocate_global_component_address().into()),
            }
            .instantiate()
            .globalize()
        }

        pub fn create_with_smuggled_given_address(
            address: GlobalAddress,
        ) -> Global<PreallocationSmugglerComponentComponent> {
            Self {
                preallocated_address: Some(address),
            }
            .instantiate()
            .globalize()
        }

        pub fn create_with_smuggled_given_address_bytes(
            preallocated_address_bytes: [u8; 30],
        ) -> Global<PreallocationSmugglerComponentComponent> {
            let address = unsafe { GlobalAddress::new_unchecked(preallocated_address_bytes) };
            Self {
                preallocated_address: Some(address),
            }
            .instantiate()
            .globalize()
        }

        pub fn smuggle_given_address(&mut self, address: GlobalAddress) {
            self.preallocated_address = Some(address);
        }

        pub fn allocate_and_smuggle_address(&mut self) {
            self.preallocated_address =
                Some(Runtime::preallocate_global_component_address().into());
        }

        pub fn instantiate_with_smuggled_address(
            &self,
        ) -> Global<PreallocationSmugglerComponentComponent> {
            let component_address = unsafe {
                ComponentAddress::new_unchecked(self.preallocated_address.unwrap().as_node_id().0)
            };
            Self {
                preallocated_address: None,
            }
            .instantiate()
            .attach_address(component_address)
            .globalize()
        }
    }
}
