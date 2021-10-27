use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn deposit_test() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!());

    // we can't use ResourceBuilder in tests, so this is a workaround to create resources
    let create_token_b_tx = TransactionBuilder::new(&executor)
        .call_function(package, "Token", "new", vec!["Token B".to_string(), "tokenB".to_string()], None)
        .build()
        .unwrap();
    let create_token_b_tx_receipt = executor.run(create_token_b_tx, false);
    let token_b_address = create_token_b_tx_receipt.component(0).unwrap();

    let create_token_c_tx = TransactionBuilder::new(&executor)
        .call_function(package, "Token", "new", vec!["Token C".to_string(), "tokenC".to_string()], None)
        .build()
        .unwrap();
    let create_token_c_tx_receipt = executor.run(create_token_c_tx, false);
    let token_c_address: Address = create_token_c_tx_receipt.component(0).unwrap();

    let transaction3 = TransactionBuilder::new(&executor)
        .call_method(token_b_address, "get_vault_amount", vec![], None)
        .build()
        .unwrap();
    let receipt3 = executor.run(transaction3, false);

    println!("{:?}\n", receipt3.results[0]);

    let create_auto_lend_tx = TransactionBuilder::new(&executor)
        .call_function(
            package, 
            "AutoLend", 
            "new", 
            vec![token_b_address.to_string(), token_c_address.to_string()], None
        )
        .build()
        .unwrap();
    let create_auto_lend_tx_receipt = executor.run(create_auto_lend_tx, false);
    let auto_lend_address = create_auto_lend_tx_receipt.component(0).unwrap();

    // let bid: Bid = TransactionBuilder::new(&executor).reserve_bucket_id();
    // let bucket = TransactionBuilder::new(&executor)
    //     .create_bucket(Amount::from(100), token_b_address, bid);

    // let transaction3 = TransactionBuilder::new(&executor)
    //     .call_method(auto_lend_address, "deposit", vec![bucket], None)
    //     .build()
    //     .unwrap();
    // let receipt3 = executor.run(transaction3, false);
}
