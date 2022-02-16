use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_hello() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor
        .publish_package(include_code!("hello_nft"))
        .unwrap();

    // Test the `instantiate_hello_nft` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "HelloNft", "instantiate_hello_nft", vec!["5".to_owned()], None)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `buy_ticket_by_id` method.
    let component = receipt1.component(0).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            component,
            "buy_ticket_by_id",
            vec![
                "328550132818421743010213967181621860355".to_owned(),
                "10,030000000000000000000000000000000000000000000000000004".to_owned(),
            ],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
