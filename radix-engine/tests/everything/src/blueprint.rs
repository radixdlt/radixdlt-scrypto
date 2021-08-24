use scrypto::constructs::*;
use scrypto::*;

blueprint! {
    struct BlueprintTest {
    }

    impl BlueprintTest {
        pub fn call_blueprint() -> Component {
            let package = Package::new(include_bytes!("helloworld.wasm"));
            let blueprint = Blueprint::from(package.into(), "Greeting");
            blueprint.call("new", args!())
        }
    }
}
