#[rustfmt::skip]
mod util;

use crate::util::TestUtil;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let auth = TestUtil::create_non_fungible_resource(&mut executor, account.clone());
    let auth_id = NonFungibleId::from(1);
    let package = TestUtil::publish_package(&mut executor, "component");
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "CrossComponent",
            "create_component_with_auth",
            vec![auth.to_string(), auth_id.to_string()],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
    let secured_component = receipt.new_component_ids[0];

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "CrossComponent", "create_component", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_ids[0];

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_method(
            my_component,
            "cross_component_call",
            vec![secured_component.to_string()],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be error");
    assert_eq!(runtime_error, RuntimeError::NotAuthorized);
}

#[test]
fn can_make_cross_component_call_with_authorization() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let auth = TestUtil::create_non_fungible_resource(&mut executor, account.clone());
    let auth_id = NonFungibleId::from(1);
    let package = TestUtil::publish_package(&mut executor, "component");
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "CrossComponent",
            "create_component_with_auth",
            vec![auth.to_string(), auth_id.to_string()],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
    let secured_component = receipt.new_component_ids[0];

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "CrossComponent", "create_component", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_ids[0];

    let auth_amount = ResourceSpecifier::Some(
        Amount::NonFungible {
            ids: BTreeSet::from([auth_id]),
        },
        auth,
    );
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&auth_amount, account)
        .call_method_with_all_resources(my_component, "put_auth")
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_method(
            my_component,
            "cross_component_call",
            vec![secured_component.to_string()],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}
