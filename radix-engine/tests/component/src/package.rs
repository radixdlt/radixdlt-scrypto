use scrypto::prelude::*;

blueprint! {
    struct PackageTest;

    impl PackageTest {
        pub fn publish() -> PackageId {
            publish_package(include_bytes!("../../../../assets/system.wasm"))
        }
    }
}
