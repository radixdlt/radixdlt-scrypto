use radix_engine::engine::*;
use scrypto::prelude::*;

#[test]
fn test_greeting() {
    // Create an in-memory Radix Engine.
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);

    // Publish this package.
    let package = proc.publish(package_code!()).unwrap();

    // Invoke the `new` function.
    let component: Address = proc
        .call_function(package, "Greeting", "new", args!())
        .and_then(decode_return)
        .unwrap();

    // Invoke the `say_hello` function.
    let rtn: u32 = proc
        .call_method(component, "say_hello", args!())
        .and_then(decode_return)
        .unwrap();
    assert_eq!(1, rtn);
}
