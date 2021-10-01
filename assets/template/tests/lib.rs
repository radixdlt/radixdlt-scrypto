use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Create an in-memory Radix Engine.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    // Create account and publish this package.
    let account = executor
        .run(
            TransactionBuilder::new().new_account().build().unwrap(),
            true,
        )
        .component(0)
        .unwrap();
    let package = executor
        .run(
            TransactionBuilder::new()
                .publish_package(package_code!())
                .build()
                .unwrap(),
            false,
        )
        .package(0)
        .unwrap();
    let abi = executor.export_abi(package, "Hello", false).unwrap();

    // Invoke the `new` function.
    let transaction = TransactionBuilder::new()
        .call_function(&abi, "new", vec![], None)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);

    // Invoke the `airdrop` function.
    let component = receipt.component(0).unwrap();
    let transaction2 = TransactionBuilder::new()
        .call_method(&abi, component, "airdrop", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, true);
    assert!(receipt2.success);
}
