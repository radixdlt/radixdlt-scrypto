use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("magic_card"));

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "HelloNft", "new", vec![], None)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, false).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `buy_special_card` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "buy_special_card",
            vec!["2".to_owned(), format!("666,{}", RADIX_TOKEN)],
            Some(account),
        )
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2, false).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);

    // Test the `buy_special_card` method.
    let component = receipt1.component(0).unwrap();
    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "buy_random_card",
            vec![format!("1000,{}", RADIX_TOKEN)],
            Some(account),
        )
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt3 = executor.run(transaction3, false).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.success);
}
