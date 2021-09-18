use radix_engine::engine::InMemoryRadixEngine;
use scrypto::prelude::*;

#[test]
fn test_component() {
    let mut engine = InMemoryRadixEngine::new(true);
    let package = engine.publish(package_code!("./everything")).unwrap();

    let component = engine
        .call_function::<Address>(package, "ComponentTest", "create_component", args!())
        .unwrap();

    engine
        .call_method::<ComponentInfo>(component, "get_component_info", args!())
        .unwrap();

    engine
        .call_method::<String>(component, "get_component_state", args!())
        .unwrap();

    engine
        .call_method::<()>(component, "put_component_state", args!())
        .unwrap();
}

#[test]
fn test_storage() {
    let mut engine = InMemoryRadixEngine::new(true);
    let package = engine.publish(package_code!("./everything")).unwrap();

    let result = engine
        .call_function::<Option<String>>(package, "StorageTest", "test_storage", args!())
        .unwrap();
    assert_eq!(Some("world".to_owned()), result)
}

#[test]
fn test_resource() {
    let mut engine = InMemoryRadixEngine::new(true);
    let package = engine.publish(package_code!("./everything")).unwrap();

    engine
        .call_function::<Bucket>(package, "ResourceTest", "create_mutable", args!())
        .unwrap();

    engine
        .call_function::<Bucket>(package, "ResourceTest", "create_fixed", args!())
        .unwrap();

    engine
        .call_function::<ResourceInfo>(package, "ResourceTest", "query", args!())
        .unwrap();
}

#[test]
fn test_bucket() {
    let mut engine = InMemoryRadixEngine::new(true);
    let package = engine.publish(package_code!("./everything")).unwrap();

    engine
        .call_function::<Bucket>(package, "BucketTest", "combine", args!())
        .unwrap();

    engine
        .call_function::<(Bucket, Bucket)>(package, "BucketTest", "split", args!())
        .unwrap();

    engine
        .call_function::<Bucket>(package, "BucketTest", "borrow", args!())
        .unwrap();

    engine
        .call_function::<(Amount, Address, Bucket)>(package, "BucketTest", "query", args!())
        .unwrap();
}

#[test]
fn test_move_resource() {
    let mut engine = InMemoryRadixEngine::new(true);
    let package = engine.publish(package_code!("./everything")).unwrap();

    engine
        .call_function::<()>(package, "MoveTest", "move_bucket", args!())
        .unwrap();

    engine
        .call_function::<Bucket>(package, "MoveTest", "move_reference", args!())
        .unwrap();
}
