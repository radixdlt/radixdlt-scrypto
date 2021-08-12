use scrypto::constructs::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct BlueprintTest {
    }

    impl BlueprintTest {
        pub fn publish() -> Address {
            Blueprint::new(include_bytes!("helloworld.wasm")).into()
        }

        pub fn invoke(blueprint: Address) -> Address {
            call_blueprint!(Address, blueprint, "Greeting", "new")
        }
    }
}
