use scrypto::prelude::*;

blueprint! {
    struct LocalComponent {
        secret: u32,
    }

    impl LocalComponent {
        pub fn get_secret(&self) -> u32 {
            self.secret
        }

        pub fn set_secret(&mut self, next: u32) {
            self.secret = next;
        }

        pub fn new(secret: u32) -> component::LocalComponent {
            Self { secret }.instantiate()
        }

        pub fn try_to_read_local_component_with_auth(
            some_non_fungible: NonFungibleAddress,
        ) -> ComponentAddress {
            let mut local_component = Self::new(12345);
            local_component.add_access_check(AccessRules::new().default(rule!(require(some_non_fungible))));

            let rtn = local_component.get_secret();
            assert_eq!(12345, rtn);

            local_component.globalize()
        }

        pub fn read_local_component() -> ComponentAddress {
            let local_component = Self::new(12345);

            let rtn = local_component.get_secret();
            assert_eq!(12345, rtn);

            local_component.globalize()
        }

        pub fn write_local_component() -> ComponentAddress {
            let local_component = Self::new(12345);
            local_component.set_secret(99999u32);
            let rtn = local_component.get_secret();
            assert_eq!(99999, rtn);

            local_component.globalize()
        }

        pub fn check_info_of_local_component(
            expected_package_address: PackageAddress,
            expected_blueprint_name: String,
        ) -> ComponentAddress {
            let local_component = Self::new(12345);

            assert_eq!(
                local_component.package_address(),
                expected_package_address
            );
            assert_eq!(
                local_component.blueprint_name(),
                expected_blueprint_name
            );

            local_component.globalize()
        }
    }
}
