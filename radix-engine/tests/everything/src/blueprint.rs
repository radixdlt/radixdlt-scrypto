use scrypto::constructs::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct BlueprintTest {
    }

    impl BlueprintTest {
        pub fn publish() -> Address {
            Package::new(include_bytes!("helloworld.wasm")).into()
        }

        pub fn invoke(package: Address) -> Address {
            let b = Blueprint::from(package, "Greeting");
            b.invoke("new", args!())
        }
    }
}
