use scrypto::prelude::*;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish() -> PackageAddress {
            component_system().publish_package(include_bytes!("../../../../assets/system.wasm"))
        }
    }
}
