use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_component() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor
        .run(
            TransactionBuilder::new().new_account().build().unwrap(),
            false,
        )
        .nth_component(0)
        .unwrap();
    let package = executor
        .run(
            TransactionBuilder::new()
                .publish_package(package_code!("./everything"))
                .build()
                .unwrap(),
            false,
        )
        .nth_package(0)
        .unwrap();
    let abi = executor
        .export_abi(package, "ComponentTest", false)
        .unwrap();

    // Create component
    let transaction1 = TransactionBuilder::new()
        .call_function(&abi, "create_component", vec![], Some(account))
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, true);
    assert!(receipt1.success);

    // Find the component address from receipt
    let component = receipt1.nth_component(0).unwrap();

    // Call functions & methods
    let transaction2 = TransactionBuilder::new()
        .call_function(
            &abi,
            "get_component_blueprint",
            vec![component.to_string()],
            Some(account),
        )
        .call_method(
            &abi,
            component,
            "get_component_state",
            vec![],
            Some(account),
        )
        .call_method(
            &abi,
            component,
            "put_component_state",
            vec![],
            Some(account),
        )
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, true);
    assert!(receipt2.success);
}

#[test]
fn test_lazy_map() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor
        .run(
            TransactionBuilder::new().new_account().build().unwrap(),
            false,
        )
        .nth_component(0)
        .unwrap();
    let package = executor
        .run(
            TransactionBuilder::new()
                .publish_package(package_code!("./everything"))
                .build()
                .unwrap(),
            false,
        )
        .nth_package(0)
        .unwrap();
    let abi = executor.export_abi(package, "LazyMapTest", false).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(&abi, "test_lazy_map", vec![], Some(account))
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}

#[test]
fn test_resource() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor
        .run(
            TransactionBuilder::new().new_account().build().unwrap(),
            false,
        )
        .nth_component(0)
        .unwrap();
    let package = executor
        .run(
            TransactionBuilder::new()
                .publish_package(package_code!("./everything"))
                .build()
                .unwrap(),
            false,
        )
        .nth_package(0)
        .unwrap();
    let abi = executor.export_abi(package, "ResourceTest", false).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(&abi, "create_mutable", vec![], Some(account))
        .call_function(&abi, "create_fixed", vec![], Some(account))
        .call_function(&abi, "query", vec![], Some(account))
        .call_function(&abi, "burn", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}

#[test]
fn test_bucket() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor
        .run(
            TransactionBuilder::new().new_account().build().unwrap(),
            false,
        )
        .nth_component(0)
        .unwrap();
    let package = executor
        .run(
            TransactionBuilder::new()
                .publish_package(package_code!("./everything"))
                .build()
                .unwrap(),
            false,
        )
        .nth_package(0)
        .unwrap();
    let abi = executor.export_abi(package, "BucketTest", false).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(&abi, "combine", vec![], Some(account))
        .call_function(&abi, "split", vec![], Some(account))
        .call_function(&abi, "borrow", vec![], Some(account))
        .call_function(&abi, "query", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}

#[test]
fn test_move_bucket_and_ref() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor
        .run(
            TransactionBuilder::new().new_account().build().unwrap(),
            false,
        )
        .nth_component(0)
        .unwrap();
    let package = executor
        .run(
            TransactionBuilder::new()
                .publish_package(package_code!("./everything"))
                .build()
                .unwrap(),
            false,
        )
        .nth_package(0)
        .unwrap();
    let abi = executor.export_abi(package, "MoveTest", false).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(&abi, "move_bucket", vec![], Some(account))
        .call_function(&abi, "move_reference", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}
