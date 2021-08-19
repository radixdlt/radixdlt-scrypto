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
            let b = Blueprint::from(blueprint);
            b.invoke("Greeting", "new", args!())
        }
    }
}
