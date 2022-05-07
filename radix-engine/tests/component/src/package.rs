use scrypto::prelude::*;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish() -> PackageAddress {
            let package = Package::new(include_bytes!("../../../../assets/system.wasm").to_vec());
            component_system().publish_package(package)
        }
    }
}
