#[rustfmt::skip]
pub mod test_runner;

use scrypto::call_data;
use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn can_create_clone_and_drop_bucket_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function_with_abi(
            package_address,
            "BucketProof",
            "create_clone_drop_bucket_proof",
            vec![format!("1,{}", resource_address), "1".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketProof"),
        )
        .unwrap()
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn can_create_clone_and_drop_vault_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_address,
            call_data!(create_clone_drop_vault_proof(Decimal::one())),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_amount() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method_with_abi(
            component_address,
            "create_clone_drop_vault_proof_by_amount",
            vec!["3".to_owned(), "1".to_owned()],
            None,
            &test_runner.export_abi_by_component(component_address),
        )
        .unwrap()
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_ids() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let total_ids = BTreeSet::from([
        NonFungibleId::from_u32(1),
        NonFungibleId::from_u32(2),
        NonFungibleId::from_u32(3),
    ]);
    let proof_ids = BTreeSet::from([NonFungibleId::from_u32(2)]);
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_address,
            call_data!(create_clone_drop_vault_proof_by_ids(total_ids, proof_ids)),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn can_use_bucket_for_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function_with_abi(
            package_address,
            "BucketProof",
            "use_bucket_proof_for_auth",
            vec![
                format!("1,{}", auth_resource_address),
                format!("1,{}", burnable_resource_address),
            ],
            Some(account),
            &test_runner.export_abi(package_address, "BucketProof"),
        )
        .unwrap()
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn can_use_vault_for_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", auth_resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method_with_abi(
            component_address,
            "use_vault_proof_for_auth",
            vec![format!("1,{}", burnable_resource_address)],
            Some(account),
            &test_runner.export_abi_by_component(component_address),
        )
        .unwrap()
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn can_create_proof_from_account_and_pass_on() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function_with_abi(
            package_address,
            "VaultProof",
            "receive_proof",
            vec![format!("1,{}", resource_address), "1".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "VaultProof"),
        )
        .unwrap()
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn cant_move_restricted_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function_with_abi(
            package_address,
            "VaultProof",
            "receive_proof_and_push_to_auth_zone",
            vec![format!("1,{}", resource_address), "1".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "VaultProof"),
        )
        .unwrap()
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::CantMoveRestrictedProof(1025))
    );
}

#[test]
fn can_compose_bucket_and_vault_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_amount(99.into(), resource_address, account)
        .take_from_worktop_by_amount(99.into(), resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                call_data!(compose_vault_and_bucket_proof(Bucket(bucket_id))),
            )
        })
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn can_compose_bucket_and_vault_proof_by_amount() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_amount(99.into(), resource_address, account)
        .take_from_worktop_by_amount(99.into(), resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                call_data!(compose_vault_and_bucket_proof_by_amount(Bucket(bucket_id), Decimal::from(2))),
            )
        })
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_compose_bucket_and_vault_proof_by_ids() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)]),
            resource_address,
            account,
        )
        .take_from_worktop_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)]),
            resource_address,
            |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    call_data!(
                        compose_vault_and_bucket_proof_by_ids(
                            Bucket(bucket_id),
                            BTreeSet::from([NonFungibleId::from_u32(1), NonFungibleId::from_u32(2),])
                        )
                    ),
                )
            },
        )
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_create_vault_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_address)],
        account,
        pk,
        &sk,
    );

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            component_address,
            call_data![create_clone_drop_vault_proof_by_amount(Decimal::from(3), Decimal::from(1))],
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_create_auth_zone_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.publish_package("proof");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .create_proof_from_account_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(1), NonFungibleId::from_u32(2)]),
            resource_address,
            account,
        )
        .create_proof_from_account_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(3)]),
            resource_address,
            account,
        )
        .create_proof_from_auth_zone_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)]),
            resource_address,
            |builder, proof_id| {
                builder.call_function(
                    package_address,
                    "Receiver",
                    call_data!(
                        assert_ids(
                            Proof(proof_id),
                            BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)]),
                            resource_address
                        )
                    ),
                )
            },
        )
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert!(receipt.result.is_ok());
}
