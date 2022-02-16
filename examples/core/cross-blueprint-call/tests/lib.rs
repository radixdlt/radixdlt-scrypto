use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_proxy_1() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor
        .publish_package(include_code!("cross_blueprint_call"))
        .unwrap();

    // Airdrop blueprint.
    executor.overwrite_package(
        Address::from_str("01bda8686d6c2fa45dce04fac71a09b54efbc8028c23aac74bc00e").unwrap(),
        include_code!("cross_blueprint_call"),
    );

    // Test the `instantiate_proxy` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Proxy1", "instantiate_proxy", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `get_gumball` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "free_token", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn test_proxy_2() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor
        .publish_package(include_code!("cross_blueprint_call"))
        .unwrap();

    // Airdrop blueprint.
    executor.overwrite_package(
        Address::from_str("01bda8686d6c2fa45dce04fac71a09b54efbc8028c23aac74bc00e").unwrap(),
        include_code!("cross_blueprint_call"),
    );

    // Test the `instantiate_proxy` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Proxy2", "instantiate_proxy", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `get_gumball` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "free_token", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
