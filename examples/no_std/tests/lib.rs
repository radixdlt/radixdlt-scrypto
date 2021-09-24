use radix_engine::engine::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Create an in-memory Radix Engine.
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);

    // Publish this package.
    let package = proc.publish(package_code!()).unwrap();

    // Invoke the `new` function.
    let component: Address = proc
        .call_function((package, "Hello".to_owned()), "new", args!())
        .and_then(decode_return)
        .unwrap();

    // Invoke the `airdrop` function.
    let _rtn: Bucket = proc
        .call_method(component, "airdrop", args!())
        .and_then(decode_return)
        .unwrap();
    // FIXME assert_eq!(1, rtn.amount());
}
