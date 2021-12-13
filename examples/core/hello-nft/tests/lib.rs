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
    let package = executor.publish_package(include_code!());

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "HelloNft", "new", vec!["5".to_owned()], None)
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, true).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `buy_ticket_by_id` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "buy_ticket_by_id",
            vec![
                "19263377484785923007266988645735551278".to_owned(),
                "10,030000000000000000000000000000000000000000000000000004".to_owned(),
            ],
            Some(account),
        )
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2, true).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);
}
