use radix_engine::engine::*;
use scrypto::prelude::*;

#[test]
fn test_greeting() {
    // Create an in-memory radix engine
    let mut engine = InMemoryRadixEngine::new(true);

    // Publish this package
    let package = engine.publish(package_code!()).unwrap();
    println!("Package address: {}", package);

    // Invoke function `Greeting::new`
    let component = engine
        .call_function::<Address>(package, "Greeting", "new", args!())
        .unwrap();
    println!("Component address: {}", component);

    // Invoke method `Greeting::say_hello`
    engine
        .call_method::<()>(component, "say_hello", args!())
        .unwrap();

    // Invoke method `Greeting::get_count`
    let count = engine
        .call_method::<u32>(component, "get_count", args!())
        .unwrap();
    assert_eq!(1, count);
}
