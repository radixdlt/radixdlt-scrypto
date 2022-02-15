use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor
        .publish_package(include_code!("hello_world"))
        .unwrap();

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Hello", "instantiate_hello", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `free_token` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "free_token", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
