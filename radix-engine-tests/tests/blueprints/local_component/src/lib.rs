use scrypto::prelude::*;

#[blueprint]
mod secret {
    struct Secret {
        secret: u32,
    }

    impl Secret {
        pub fn get_secret(&self) -> u32 {
            self.secret
        }

        pub fn set_secret(&mut self, next: u32) {
            self.secret = next;
        }

        pub fn new(secret: u32) -> SecretComponent {
            Self { secret }.instantiate().own()
        }

        pub fn read_local_component() -> Global<SecretComponent> {
            let local_component = Self { secret: 12345 }.instantiate();

            let rtn = local_component.get_secret();
            assert_eq!(12345, rtn);

            local_component.globalize()
        }

        pub fn write_local_component() -> Global<SecretComponent> {
            let local_component = Self { secret: 12345 }.instantiate();

            local_component.set_secret(99999u32);
            let rtn = local_component.get_secret();
            assert_eq!(99999, rtn);

            local_component.globalize()
        }

        pub fn check_info_of_local_component(
            expected_package_address: PackageAddress,
            expected_blueprint_name: String,
        ) -> Global<SecretComponent> {
            let local_component = Self { secret: 12345 }.instantiate();

            let blueprint = local_component.blueprint();

            assert_eq!(blueprint.package_address, expected_package_address);
            assert_eq!(blueprint.blueprint_name, expected_blueprint_name);

            local_component.globalize()
        }
    }
}

#[blueprint]
mod stored_kv_local {
    use secret::*;

    struct StoredKVLocal {
        components: KeyValueStore<u32, SecretComponent>,
    }

    impl StoredKVLocal {
        pub fn parent_get_secret(&self) -> u32 {
            self.components.get(&0u32).unwrap().get_secret()
        }

        pub fn parent_set_secret(&mut self, next: u32) {
            self.components.get(&0u32).unwrap().set_secret(next)
        }

        fn new_internal(secret: u32) -> Globalizeable<StoredKVLocalComponent> {
            let component = SecretComponent::new(secret);
            let components = KeyValueStore::new();
            components.insert(0u32, component);

            Self { components }.instantiate()
        }

        pub fn new(secret: u32) -> StoredKVLocalComponent {
            Self::new_internal(secret).own()
        }

        pub fn new_global(secret: u32) -> Global<StoredKVLocalComponent> {
            Self::new_internal(secret).globalize()
        }

        pub fn call_read_on_stored_component_in_owned_component() -> Global<StoredKVLocalComponent>
        {
            let my_component = Self::new_internal(12345);

            let rtn = my_component.parent_get_secret();
            assert_eq!(12345, rtn);

            my_component.globalize()
        }

        pub fn call_write_on_stored_component_in_owned_component() -> Global<StoredKVLocalComponent>
        {
            let my_component = Self::new_internal(12345);

            my_component.parent_set_secret(99999);
            let rtn = my_component.parent_get_secret();
            assert_eq!(99999, rtn);

            my_component.globalize()
        }
    }
}

#[blueprint]
mod stored_secret {
    use secret::*;

    struct StoredSecret {
        component: SecretComponent,
    }

    impl StoredSecret {
        pub fn parent_get_secret(&self) -> u32 {
            self.component.get_secret()
        }

        pub fn parent_set_secret(&mut self, next: u32) {
            self.component.set_secret(next)
        }

        pub fn new(secret: u32) -> StoredSecretComponent {
            let component = SecretComponent::new(secret);
            Self { component }.instantiate().own()
        }

        pub fn new_global(secret: u32) -> Global<StoredSecretComponent> {
            let component = SecretComponent::new(secret);
            Self { component }.instantiate().globalize()
        }

        pub fn call_read_on_stored_component_in_owned_component() -> Global<StoredSecretComponent> {
            let component = SecretComponent::new(12345);
            let my_component = Self { component }.instantiate();

            let rtn = my_component.parent_get_secret();
            assert_eq!(12345, rtn);

            my_component.globalize()
        }

        pub fn call_write_on_stored_component_in_owned_component() -> Global<StoredSecretComponent>
        {
            let component = SecretComponent::new(12345);
            let my_component = Self { component }.instantiate();

            my_component.parent_set_secret(99999);
            let rtn = my_component.parent_get_secret();
            assert_eq!(99999, rtn);

            my_component.globalize()
        }
    }
}
