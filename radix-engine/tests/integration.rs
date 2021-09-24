use radix_engine::engine::InMemoryRadixEngine;
use scrypto::prelude::*;

#[test]
fn test_component() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    let component: Address = scrypto_decode(
        &proc
            .call_function(
                (pkg, "ComponentTest".to_owned()),
                "create_component",
                args!(),
            )
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
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    let result: Option<String> = scrypto_decode(
        &proc
            .call_function((pkg, "StorageTest".to_owned()), "test_storage", args!())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(Some("world".to_owned()), result)
}

#[test]
fn test_resource() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    proc.call_function((pkg, "ResourceTest".to_owned()), "create_mutable", args!())
        .unwrap();
    proc.call_function((pkg, "ResourceTest".to_owned()), "create_fixed", args!())
        .unwrap();
    proc.call_function((pkg, "ResourceTest".to_owned()), "query", args!())
        .unwrap();
}

#[test]
fn test_bucket() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    proc.call_function((pkg, "BucketTest".to_owned()), "combine", args!())
        .unwrap();
    proc.call_function((pkg, "BucketTest".to_owned()), "split", args!())
        .unwrap();
    proc.call_function((pkg, "BucketTest".to_owned()), "borrow", args!())
        .unwrap();
    proc.call_function((pkg, "BucketTest".to_owned()), "query", args!())
        .unwrap();
}

#[test]
fn test_move_resource() {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);
    let pkg = proc.publish(package_code!("./everything")).unwrap();

    proc.call_function((pkg, "MoveTest".to_owned()), "move_bucket", args!())
        .unwrap();
    proc.call_function((pkg, "MoveTest".to_owned()), "move_reference", args!())
        .unwrap();
}
