use radix_engine::engine::*;
use scrypto::prelude::*;

#[test]
fn test_greeting() {
    // Create an in-memory Radix Engine.
    let mut engine = InMemoryRadixEngine::new(true);

    // Publish this package.
    let package = engine.publish(package_code!()).unwrap();

    // Invoke the new function.
    let component = engine
        .call_function::<Address>(package, "Greeting", "new", args!())
        .unwrap();

    // Invoke the `say_hello` function.
    let rtn = engine
        .call_method::<u32>(component, "say_hello", args!())
        .unwrap();
    assert_eq!(1, rtn);
}
