use scrypto::prelude::*;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish_package() -> Package {
            Package::new(include_bytes!("../../../../assets/system.wasm"))
        }
    }
}
