#[rustfmt::skip]
mod util;

use crate::util::TestUtil;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_package() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let package = TestUtil::publish_package(&mut executor, "component");

    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "PackageTest", "publish", vec![], Some(account))
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_component() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let package = TestUtil::publish_package(&mut executor, "component");

    // Create component
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ComponentTest",
            "create_component",
            vec![],
            Some(account),
        )
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    assert!(receipt1.result.is_ok());

    // Find the component ID from receipt
    let component = receipt1.new_component_ids[0];

    // Call functions & methods
    let transaction2 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            vec![component.to_string()],
            Some(account),
        )
        .call_method(component, "get_component_state", vec![], Some(account))
        .call_method(component, "put_component_state", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    assert!(receipt2.result.is_ok());
}
