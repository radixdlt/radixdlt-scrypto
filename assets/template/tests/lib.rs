use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!("${wasm_name}")).unwrap();

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Hello", "instantiate_hello", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(&[pk])
        .unwrap()
        .sign(&[sk]) ;
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `free_token` method.
    let component = receipt1.new_component_ids[0];
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "free_token", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(&[pk])
        .unwrap()
        .sign(&[sk]) ;
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
