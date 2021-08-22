use scrypto::constructs::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct PackageTest {
    }

    impl PackageTest {
        pub fn publish() -> Address {
            Package::new(include_bytes!("helloworld.wasm")).into()
        }
    }
}
