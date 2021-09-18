use radix_engine::engine::InMemoryRadixEngine;
use scrypto::prelude::*;

#[test]
fn test_component() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_runtime();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    let component: Address = scrypto_decode(
        &proc
            .call_function(pkg, "ComponentTest", "create_component", args!())
            .unwrap(),
    )
    .unwrap();
    proc.call_method(component, "get_component_info", args!())
        .unwrap();
    proc.call_method(component, "get_component_state", args!())
        .unwrap();
    proc.call_method(component, "put_component_state", args!())
        .unwrap();
}

#[test]
fn test_storage() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_runtime();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    let result: Option<String> = scrypto_decode(
        &proc
            .call_function(pkg, "StorageTest", "test_storage", args!())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(Some("world".to_owned()), result)
}

#[test]
fn test_resource() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_runtime();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    proc.call_function(pkg, "ResourceTest", "create_mutable", args!())
        .unwrap();
    proc.call_function(pkg, "ResourceTest", "create_fixed", args!())
        .unwrap();
    proc.call_function(pkg, "ResourceTest", "query", args!())
        .unwrap();
}

#[test]
fn test_bucket() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_runtime();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    proc.call_function(pkg, "BucketTest", "combine", args!())
        .unwrap();
    proc.call_function(pkg, "BucketTest", "split", args!())
        .unwrap();
    proc.call_function(pkg, "BucketTest", "borrow", args!())
        .unwrap();
    proc.call_function(pkg, "BucketTest", "query", args!())
        .unwrap();
}

#[test]
fn test_move_resource() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_runtime();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    proc.call_function(pkg, "MoveTest", "move_bucket", args!())
        .unwrap();
    proc.call_function(pkg, "MoveTest", "move_reference", args!())
        .unwrap();
}
