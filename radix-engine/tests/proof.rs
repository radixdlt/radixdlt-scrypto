#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

fn instantiate_vault_proof_component(
    test_runner: &mut TestRunner,
    package_id: PackageId,
    resource_def_id: ResourceDefId,
    account: ComponentId,
    key: EcdsaPublicKey,
) -> ComponentId {
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_id,
            "VaultProof",
            "new",
            vec![format!("1,{}", resource_def_id)],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    receipt.new_component_ids[0]
}

#[test]
fn can_create_clone_and_drop_bucket_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_non_fungible_resource(account);
    let package_id = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_id,
            "BucketProof",
            "create_clone_drop_bucket_proof",
            vec![format!("1,{}", resource_def_id), "1".to_owned()],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_create_clone_and_drop_vault_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_non_fungible_resource(account);
    let package_id = test_runner.publish_package("proof");
    let component_id = instantiate_vault_proof_component(
        &mut test_runner,
        package_id,
        resource_def_id,
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_id,
            "create_clone_drop_vault_proof",
            vec!["1".to_owned()],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_use_bucket_for_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (auth_resource_def_id, burnable_resource_def_id) =
        test_runner.create_restricted_burn_token(account);
    let package_id = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_id,
            "BucketProof",
            "use_bucket_proof_for_auth",
            vec![
                format!("1,{}", auth_resource_def_id),
                format!("1,{}", burnable_resource_def_id),
            ],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_use_vault_for_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (auth_resource_def_id, burnable_resource_def_id) =
        test_runner.create_restricted_burn_token(account);
    let package_id = test_runner.publish_package("proof");
    let component_id = instantiate_vault_proof_component(
        &mut test_runner,
        package_id,
        auth_resource_def_id,
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_id,
            "use_vault_proof_for_auth",
            vec![format!("1,{}", burnable_resource_def_id)],
            Some(account),
        )
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}
