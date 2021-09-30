use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_component() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.new_account(false);
    let package = executor.publish_package(package_code!("./everything"), false);
    let abi = executor
        .export_abi(package, "ComponentTest", false)
        .unwrap();

    // Create component
    let transaction1 = TransactionBuilder::new()
        .call_function(&abi, "create_component", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt1 = executor.execute(&transaction1, true);
    assert!(receipt1.success);

    // Find the component address from receipt
    let component = receipt1.nth_component(0).unwrap();

    // Call functions & methods
    let txn2 = TransactionBuilder::new()
        .call_function(
            &abi,
            "get_component_blueprint",
            vec![component.to_string().as_ref()],
        )
        .call_method(&abi, component, "get_component_state", vec![])
        .call_method(&abi, component, "put_component_state", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt2 = executor.execute(&txn2, true);
    assert!(receipt2.success);
}

#[test]
fn test_lazy_map() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.new_account(false);
    let package = executor.publish_package(package_code!("./everything"), false);
    let abi = executor.export_abi(package, "LazyMapTest", false).unwrap();

    let txn = TransactionBuilder::new()
        .call_function(&abi, "test_lazy_map", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt = executor.execute(&txn, true);
    assert!(receipt.success);
}

#[test]
fn test_resource() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.new_account(false);
    let package = executor.publish_package(package_code!("./everything"), false);
    let abi = executor.export_abi(package, "ResourceTest", false).unwrap();

    let txn = TransactionBuilder::new()
        .call_function(&abi, "create_mutable", vec![])
        .call_function(&abi, "create_fixed", vec![])
        .call_function(&abi, "query", vec![])
        .call_function(&abi, "burn", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt = executor.execute(&txn, true);
    assert!(receipt.success);
}

#[test]
fn test_bucket() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.new_account(false);
    let package = executor.publish_package(package_code!("./everything"), false);
    let abi = executor.export_abi(package, "BucketTest", false).unwrap();

    let txn = TransactionBuilder::new()
        .call_function(&abi, "combine", vec![])
        .call_function(&abi, "split", vec![])
        .call_function(&abi, "borrow", vec![])
        .call_function(&abi, "query", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt = executor.execute(&txn, true);
    assert!(receipt.success);
}

#[test]
fn test_move_bucket_and_ref() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.new_account(false);
    let package = executor.publish_package(package_code!("./everything"), false);
    let abi = executor.export_abi(package, "MoveTest", false).unwrap();

    let txn = TransactionBuilder::new()
        .call_function(&abi, "move_bucket", vec![])
        .call_function(&abi, "move_reference", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt = executor.execute(&txn, true);
    assert!(receipt.success);
}
