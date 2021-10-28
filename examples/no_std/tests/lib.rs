use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_no_std() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let package = executor.publish_package(include_code!());

    // Test the `hello` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "NoStd", "hello", vec![], None)
        .build(Vec::new())
        .unwrap();
    let receipt1 = executor.run(transaction1, true).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);
}
