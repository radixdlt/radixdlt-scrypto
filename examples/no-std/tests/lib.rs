use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_say_hello() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let public_key = executor.new_public_key();
    let account = executor.new_account(public_key);
    let package = executor
        .publish_package(include_package!("no_std"))
        .unwrap();

    // Test the `say_hello` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "NoStd", "say_hello", vec![], Some(account))
        .build(vec![public_key])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());
}
