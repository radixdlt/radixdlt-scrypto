#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

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
        .parse_args_and_call_function(
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
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_def_id)],
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_id,
            "create_clone_drop_vault_proof",
            vec![scrypto_encode(&Decimal::from(1))],
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_amount() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), account);
    let package_id = test_runner.publish_package("proof");
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_def_id)],
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .parse_args_and_call_method(
            component_id,
            "create_clone_drop_vault_proof_by_amount",
            vec!["3".to_owned(), "1".to_owned()],
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
fn can_create_clone_and_drop_vault_proof_by_ids() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_non_fungible_resource(account);
    let package_id = test_runner.publish_package("proof");
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_def_id)],
        account,
        key,
    );

    // Act
    let total_ids = BTreeSet::from([
        NonFungibleId::from(1),
        NonFungibleId::from(2),
        NonFungibleId::from(3),
    ]);
    let proof_ids = BTreeSet::from([NonFungibleId::from(2)]);
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_id,
            "create_clone_drop_vault_proof_by_ids",
            args![total_ids, proof_ids],
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
        .parse_args_and_call_function(
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
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("1,{}", auth_resource_def_id)],
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .parse_args_and_call_method(
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

#[test]
fn can_create_proof_from_account_and_pass_on() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), account);
    let package_id = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .parse_args_and_call_function(
            package_id,
            "VaultProof",
            "receive_proof",
            vec![format!("1,{}", resource_def_id), "1".to_owned()],
            Some(account),
        )
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cant_move_restricted_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), account);
    let package_id = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .parse_args_and_call_function(
            package_id,
            "VaultProof",
            "receive_proof_and_move_to_auth_zone",
            vec![format!("1,{}", resource_def_id), "1".to_owned()],
            Some(account),
        )
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(matches!(
        receipt.result,
        Err(RuntimeError::CantMoveRestrictedProof(512))
    ));
}

#[test]
fn can_compose_bucket_and_vault_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), account);
    let package_id = test_runner.publish_package("proof");
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_def_id)],
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_amount(99.into(), resource_def_id, account)
        .take_from_worktop_by_amount(99.into(), resource_def_id, |builder, bucket_id| {
            builder.call_method(
                component_id,
                "compose_vault_and_bucket_proof",
                args![Bucket(bucket_id)],
            )
        })
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_compose_bucket_and_vault_proof_by_amount() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), account);
    let package_id = test_runner.publish_package("proof");
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_def_id)],
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_amount(99.into(), resource_def_id, account)
        .take_from_worktop_by_amount(99.into(), resource_def_id, |builder, bucket_id| {
            builder.call_method(
                component_id,
                "compose_vault_and_bucket_proof_by_amount",
                args![Bucket(bucket_id), Decimal::from(2)],
            )
        })
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_compose_bucket_and_vault_proof_by_ids() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_non_fungible_resource(account);
    let package_id = test_runner.publish_package("proof");
    let component_id = test_runner.instantiate_component(
        package_id,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_def_id)],
        account,
        key,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_ids(
            &BTreeSet::from([NonFungibleId::from(2), NonFungibleId::from(3)]),
            resource_def_id,
            account,
        )
        .take_from_worktop_by_ids(
            &BTreeSet::from([NonFungibleId::from(2), NonFungibleId::from(3)]),
            resource_def_id,
            |builder, bucket_id| {
                builder.call_method(
                    component_id,
                    "compose_vault_and_bucket_proof_by_ids",
                    args![
                        Bucket(bucket_id),
                        BTreeSet::from([NonFungibleId::from(1), NonFungibleId::from(2),])
                    ],
                )
            },
        )
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}
