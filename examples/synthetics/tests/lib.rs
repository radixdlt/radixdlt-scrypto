use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!());

    // create a price oracle
    let po_transaction = TransactionBuilder::new(&executor)
        .call_function(package, "PriceOracle", "new", vec![], None)
        .build()
        .unwrap();
    let po_receipt = executor.run(po_transaction, false);
    println!("{:?}\n", po_receipt);
    assert!(po_receipt.success);

    // create our synthetic pool
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "SyntheticPool", "new", vec![
            po_receipt.component(0).unwrap().to_string(),
            "collateral".to_string(),
            "underlying".to_string(),
            "synthetic".to_string()
            ], None)
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, false);
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `get price` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(component, "free_token", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, false);
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);
}
