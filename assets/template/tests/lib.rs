use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Create an in-memory Radix Engine.
    let mut ledger = InMemoryLedger::new();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.new_account(false);

    // Publish this package.
    let package = executor.publish_package(package_code!(), false);
    let abi = executor.export_abi(package, "Hello", false).unwrap();

    // Invoke the `new` function.
    let transaction = TransactionBuilder::new()
        .call_function(&abi, "new", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt = executor.execute(&transaction, true);
    assert!(receipt.success);

    // Read component address from the receipt.
    let component = receipt.nth_component(0).unwrap();

    // Invoke the `airdrop` function.
    let transaction2 = TransactionBuilder::new()
        .call_method(&abi, component, "airdrop", vec![])
        .build_with(Some(account))
        .unwrap();
    let receipt2 = executor.execute(&transaction2, true);
    assert!(receipt2.success);
}
