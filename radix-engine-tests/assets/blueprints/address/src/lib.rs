use scrypto::prelude::*;

#[blueprint]
mod child_component {
    use called_component::*;

    struct ChildComponent {
        to_call: Global<CalledComponent>,
    }

    impl ChildComponent {
        pub fn create(to_call: Global<CalledComponent>) -> Owned<ChildComponent> {
            Self { to_call }.instantiate()
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

        pub fn assert_check_on_global_blueprint_caller(&self, blueprint: BlueprintId) {
            Runtime::assert_access_rule(rule!(require(global_caller(blueprint))));
        }
    }
}

#[blueprint]
mod my_component {
    use called_component::*;
    use child_component::*;

    struct MyComponent {
        child: Owned<ChildComponent>,
        to_call: Global<CalledComponent>,
    }

    impl MyComponent {
        pub fn create(to_call: Global<CalledComponent>) -> Global<MyComponent> {
            let child = ChildComponent::create(to_call.clone());
            Self { child, to_call }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn get_global_address_in_parent(&self) -> ComponentAddress {
            Runtime::global_address()
        }

        pub fn get_global_address_in_owned(&self) -> ComponentAddress {
            self.child.get_global_address()
        }

        pub fn get_global_address_in_local(to_call: Global<CalledComponent>) -> ComponentAddress {
            let child = ChildComponent::create(to_call.clone());
            let address = child.get_global_address();
            Self { child, to_call }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            address
        }

        pub fn get_global_address_in_local_of_parent_method(
            &self,
            to_call: Global<CalledComponent>,
        ) -> ComponentAddress {
            Self::get_global_address_in_local(to_call)
        }

        pub fn call_other_component_with_wrong_address(&self) {
            let address = self.to_call.address();
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

        pub fn assert_check_on_global_blueprint_caller(&self, blueprint: BlueprintId, child: bool) {
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
        child: Owned<CalledComponentChild>,
    }

    impl CalledComponent {
        pub fn create() -> Global<CalledComponent> {
            let child = CalledComponentChild::create();
            Self { child }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
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
        pub fn create() -> Owned<CalledComponentChild> {
            Self {}.instantiate()
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
        pub fn create_with_preallocated_address() -> Global<PreallocationComponent> {
            let (own, _component_address) =
                Runtime::allocate_component_address(PreallocationComponent::blueprint_id());
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize()
        }

        pub fn create_with_unused_preallocated_address_1() -> Global<PreallocationComponent> {
            let (own, _component_address) =
                Runtime::allocate_component_address(PreallocationComponent::blueprint_id());
            let _ = Runtime::allocate_component_address(PreallocationComponent::blueprint_id());
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize()
        }

        pub fn create_with_unused_preallocated_address_2() -> Global<PreallocationComponent> {
            let _ = Runtime::allocate_component_address(PreallocationComponent::blueprint_id());
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn create_two_with_same_address() -> (
            Global<PreallocationComponent>,
            Global<PreallocationComponent>,
        ) {
            let (own, _component_address) =
                Runtime::allocate_component_address(PreallocationComponent::blueprint_id());
            let one = Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own.clone())
                .globalize();
            let two = Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(own)
                .globalize();
            (one, two)
        }
    }
}

#[blueprint]
mod manifest_global_addresses {
    struct ManifestGlobalAddresses {}

    impl ManifestGlobalAddresses {
        pub fn accept_global_addresses(
            a: GlobalAddress,
            b: PackageAddress,
            c: ComponentAddress,
            d: ResourceAddress,
        ) {
            info!("{:?}, {:?}, {:?}, {:?}", a, b, c, d);
        }
    }
}
