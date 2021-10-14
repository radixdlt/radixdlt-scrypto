use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test1_rename_me() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let synthetics_package = executor.publish_package(include_code!());

    let oracle_package_address = Address::from_str("01806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425").unwrap();

    executor.publish_package_to(
        include_code!("../../price-oracle"),
        oracle_package_address,
    );

    // create a price oracle
    let po_transaction = TransactionBuilder::new(&executor)
        .call_function(oracle_package_address, "PriceOracle", "new", vec![], None)
        .build()
        .unwrap();
    let po_receipt = executor.run(po_transaction, false);
    println!("{:?}\n", po_receipt);
    assert!(po_receipt.success);

    // create our synthetic pool
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(synthetics_package, "SyntheticPool", "new", vec![
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


}
