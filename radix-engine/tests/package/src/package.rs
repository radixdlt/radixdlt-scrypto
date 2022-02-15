use scrypto::prelude::*;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish_package() -> PackageRef {
            Context::publish_package(include_bytes!("../../../../assets/system.wasm"))
        }
    }
}
