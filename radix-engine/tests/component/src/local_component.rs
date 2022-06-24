use scrypto::prelude::*;

blueprint! {
    struct LocalComponent {
        secret: u32
    }

    impl LocalComponent {
        pub fn get_secret(&self) -> u32 {
            self.secret
        }

        pub fn call_local_component() -> ComponentAddress {
            let local_component = Self {
                secret: 12345
            }.instantiate();

            let rtn: u32 = local_component.call("get_secret", vec![]);
            assert_eq!(12345, rtn);

            local_component.globalize()
        }

        pub fn check_info_of_local_component(
            expected_package_address: PackageAddress,
            expected_blueprint_name: String,
        ) -> ComponentAddress {
            let local_component = Self {
                secret: 12345
            }.instantiate();

            assert_eq!(local_component.package_address(), expected_package_address);
            assert_eq!(local_component.blueprint_name(), expected_blueprint_name);

            local_component.globalize()
        }
    }
}
