use radix_engine::engine::{KernelError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::resource::{Bucket, Proof, DIVISIBILITY_MAXIMUM};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn can_create_clone_and_drop_bucket_proof() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.extract_and_publish_package("proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
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
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_clone_and_drop_vault_proof() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            component_address,
            "create_clone_drop_vault_proof",
            args!(Decimal::one()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_amount() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method_with_abi(
            component_address,
            "create_clone_drop_vault_proof_by_amount",
            vec!["3".to_owned(), "1".to_owned()],
            None,
            &test_runner.export_abi_by_component(component_address),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_ids() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let total_ids = BTreeSet::from([
        NonFungibleId::from_u32(1),
        NonFungibleId::from_u32(2),
        NonFungibleId::from_u32(3),
    ]);
    let proof_ids = BTreeSet::from([NonFungibleId::from_u32(2)]);
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            component_address,
            "create_clone_drop_vault_proof_by_ids",
            args!(total_ids, proof_ids),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_bucket_for_authorization() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.extract_and_publish_package("proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
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
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_vault_for_authorization() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", auth_resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method_with_abi(
            component_address,
            "use_vault_proof_for_auth",
            vec![format!("1,{}", burnable_resource_address)],
            Some(account),
            &test_runner.export_abi_by_component(component_address),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_proof_from_account_and_pass_on() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.extract_and_publish_package("proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function_with_abi(
            package_address,
            "VaultProof",
            "receive_proof",
            vec![format!("1,{}", resource_address), "1".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "VaultProof"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cant_move_restricted_proof() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.extract_and_publish_package("proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function_with_abi(
            package_address,
            "VaultProof",
            "receive_proof_and_push_to_auth_zone",
            vec![format!("1,{}", resource_address), "1".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "VaultProof"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CantMoveRestrictedProof)
        )
    });
}

#[test]
fn cant_move_locked_bucket() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.extract_and_publish_package("proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function_with_abi(
            package_address,
            "BucketProof",
            "return_bucket_while_locked",
            vec![format!("1,{}", resource_address), "1".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketProof"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CantMoveLockedBucket)
        )
    });
}

#[test]
fn can_compose_bucket_and_vault_proof() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(99.into(), resource_address, account)
        .take_from_worktop_by_amount(99.into(), resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof",
                args!(Bucket(bucket_id)),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof_by_amount() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(99.into(), resource_address, account)
        .take_from_worktop_by_amount(99.into(), resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof_by_amount",
                args!(Bucket(bucket_id), Decimal::from(2)),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof_by_ids() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("1,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
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
                    "compose_vault_and_bucket_proof_by_ids",
                    args!(
                        Bucket(bucket_id),
                        BTreeSet::from([NonFungibleId::from_u32(1), NonFungibleId::from_u32(2),])
                    ),
                )
            },
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_vault_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.extract_and_publish_package("proof");
    let component_address = test_runner.instantiate_component(
        package_address,
        "VaultProof",
        "new",
        vec![format!("3,{}", resource_address)],
        account,
        public_key,
    );

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            component_address,
            "create_clone_drop_vault_proof_by_amount",
            args!(Decimal::from(3), Decimal::from(1)),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_auth_zone_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.extract_and_publish_package("proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
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
                    "assert_ids",
                    args!(
                        Proof(proof_id),
                        BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)]),
                        resource_address
                    ),
                )
            },
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}
