use scrypto::prelude::*;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish_package() -> PackageRef {
            PackageRef::new(include_bytes!("../../../../assets/system.wasm"))
        }
    }
}
