use scrypto::constructs::*;
use scrypto::*;

blueprint! {
    struct BlueprintTest {
    }

    impl BlueprintTest {
        pub fn invoke_blueprint() -> Component {
            let package = Package::new(include_bytes!("helloworld.wasm"));
            let blueprint = Blueprint::from(package.into(), "Greeting");
            blueprint.invoke("new", args!())
        }
    }
}
