use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_magic_card() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("magic_card")).unwrap();

    // Test the `instantiate_component` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "HelloNft", "instantiate_component", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `buy_special_card` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "buy_special_card",
            vec![hex::encode(2u128.to_be_bytes()), format!("666,{}", RADIX_TOKEN)],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `buy_special_card` method.
    let component = receipt1.component(0).unwrap();
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "buy_random_card",
            vec![format!("1000,{}", RADIX_TOKEN)],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt3 = executor.run(transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());
}
