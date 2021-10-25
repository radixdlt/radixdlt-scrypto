use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_vendor() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!());

    // Mock the GumballMachine blueprint.
    executor.publish_package_to(
        include_code!("../../gumball-machine"),
        Address::from_str("01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876").unwrap(),
    );

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Vendor", "new", vec![], None)
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, false);
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
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, false);
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);
}

#[test]
fn test_sub_vendor() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!());

    // Mock the GumballMachine blueprint.
    executor.publish_package_to(
        include_code!("../../gumball-machine"),
        Address::from_str("01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876").unwrap(),
    );

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "SubVendor", "new", vec![], None)
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, false);
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
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, false);
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);
}
