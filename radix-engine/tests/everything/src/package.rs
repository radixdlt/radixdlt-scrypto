use scrypto::blueprint;
use scrypto::core::Package;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish_package() -> Package {
            Package::new(include_bytes!("../../../../assets/system.wasm"))
        }
    }
}
