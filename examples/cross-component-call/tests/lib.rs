use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_vendor() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());

    // Mock the GumballMachine blueprint.
    executor.overwrite_package(
        Address::from_str("01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876").unwrap(),
        include_code!("../../gumball-machine"),
    );

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Vendor", "new", vec![], None)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, true).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `get_gumball` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "get_gumball",
            vec!["1,030000000000000000000000000000000000000000000000000004".to_owned()],
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

#[test]
fn test_sub_vendor() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());

    // Mock the GumballMachine blueprint.
    executor.overwrite_package(
        Address::from_str("01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876").unwrap(),
        include_code!("../../gumball-machine"),
    );

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "SubVendor", "new", vec![], None)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, true).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `get_gumball` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "get_gumball",
            vec!["1,030000000000000000000000000000000000000000000000000004".to_owned()],
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
