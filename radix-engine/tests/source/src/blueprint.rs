use scrypto::constructs::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct BlueprintTest {
    }

    impl BlueprintTest {

        pub fn publish() -> Address {
            // the hello world example
            Blueprint::new(include_bytes!("test.wasm"))
        }

        pub fn invoke(blueprint: Address) -> Address {
            call_blueprint!(Address, blueprint, "Greeting", "new")
        }
    }
}
