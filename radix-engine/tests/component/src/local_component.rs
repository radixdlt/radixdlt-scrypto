use scrypto::prelude::*;

blueprint! {
    struct LocalComponent {}

    impl LocalComponent {
        pub fn check_info_of_local_component(
            expected_package_address: PackageAddress,
            expected_blueprint_name: String,
        ) -> ComponentAddress {
            let local_component = Self {}.instantiate();

            assert_eq!(local_component.package_address(), expected_package_address);
            assert_eq!(local_component.blueprint_name(), expected_blueprint_name);

            local_component.globalize()
        }
    }
}
