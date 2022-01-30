use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_withdraw_all() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("managed_access"));

    // Publish FlatAdmin
    executor.overwrite_package(
        "01ca59a8d6ea4f7efa1765cef702d14e47570c079aedd44992dd09"
            .parse()
            .unwrap(),
        include_code!("../../flat-admin", "flat_admin"),
    );

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "ManagedAccess", "new", vec![], None)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `withdraw_all` method.
    let managed_access = receipt1.component(1).unwrap();
    let admin_badge = receipt1.resource_def(1).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            managed_access,
            "withdraw_all",
            vec![format!("1,{}", admin_badge)],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
