use radix_common::prelude::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto::resource::DIVISIBILITY_MAXIMUM;
use scrypto_test::prelude::*;

#[test]
fn can_create_clone_and_drop_bucket_proof() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_non_fungible_resource(account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 1)
        .take_all_from_worktop(resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "BucketProof",
                "create_clone_drop_bucket_proof",
                manifest_args!(lookup.bucket("bucket"), dec!(1)),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_amount() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account(account, resource_address, 3)
                .take_all_from_worktop(resource_address, "bucket")
                .with_name_lookup(|builder, lookup| {
                    let bucket = lookup.bucket("bucket");
                    builder.call_function(
                        package_address,
                        "VaultProof",
                        "new",
                        manifest_args!(bucket),
                    )
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "create_clone_drop_vault_proof_by_amount",
            manifest_args!(dec!("3"), dec!(1)),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_clone_and_drop_vault_proof_by_ids() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_non_fungible_resource(account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account(account, resource_address, 3)
                .take_all_from_worktop(resource_address, "bucket")
                .with_name_lookup(|builder, lookup| {
                    let bucket = lookup.bucket("bucket");
                    builder.call_function(
                        package_address,
                        "VaultProof",
                        "new",
                        manifest_args!(bucket),
                    )
                })
        },
    );

    // Act
    let non_fungible_local_ids = BTreeSet::from([
        NonFungibleLocalId::integer(1),
        NonFungibleLocalId::integer(2),
        NonFungibleLocalId::integer(3),
    ]);
    let proof_non_fungible_local_ids = BTreeSet::from([NonFungibleLocalId::integer(2)]);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "create_clone_drop_vault_proof_by_ids",
            manifest_args!(non_fungible_local_ids, proof_non_fungible_local_ids),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_bucket_for_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (auth_resource_address, burnable_resource_address) =
        ledger.create_restricted_burn_token(account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, auth_resource_address, 1)
        .withdraw_from_account(account, burnable_resource_address, 1)
        .take_all_from_worktop(auth_resource_address, "auth_bucket")
        .take_all_from_worktop(burnable_resource_address, "burnable_bucket")
        .with_name_lookup(|builder, lookup| {
            let auth_bucket = lookup.bucket("auth_bucket");
            let burnable_bucket = lookup.bucket("burnable_bucket");
            builder.call_function(
                package_address,
                "BucketProof",
                "use_bucket_proof_for_auth",
                manifest_args!(auth_bucket, burnable_bucket),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_vault_for_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (auth_resource_address, burnable_resource_address) =
        ledger.create_restricted_burn_token(account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account(account, auth_resource_address, 1)
                .take_all_from_worktop(auth_resource_address, "bucket")
                .with_name_lookup(|builder, lookup| {
                    let bucket = lookup.bucket("bucket");
                    builder.call_function(
                        package_address,
                        "VaultProof",
                        "new",
                        manifest_args!(bucket),
                    )
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, burnable_resource_address, 1)
        .take_all_from_worktop(burnable_resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "use_vault_proof_for_auth",
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_proof_from_account_and_pass_on() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resource_address, 1)
        .pop_from_auth_zone("proof")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("proof");
            builder.call_function(
                package_address,
                "VaultProof",
                "receive_proof",
                manifest_args!(proof),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cant_move_restricted_proof_to_auth_zone() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resource_address, 1)
        .pop_from_auth_zone("proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "VaultProof",
                "receive_proof_and_push_to_auth_zone",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(e))
            if e.eq("Moving restricted proof downstream") =>
        {
            true
        }
        _ => false,
    });
}

#[test]
fn cant_move_restricted_proof_to_scrypto_function_aka_barrier() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resource_address, 1)
        .pop_from_auth_zone("proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "VaultProof",
                "receive_proof_and_pass_to_scrypto_function",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(e))
            if e.eq("Moving restricted proof downstream") =>
        {
            true
        }
        _ => false,
    });
}

#[test]
fn can_move_restricted_proof_to_proof_function_aka_non_barrier() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resource_address, 1)
        .pop_from_auth_zone("proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "VaultProof",
                "receive_proof_and_drop",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_move_restricted_proofs_internally() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let (public_key, _, account) = ledger.new_allocated_account();
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "Outer", "instantiate", manifest_args!())
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success().new_component_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "pass_fungible_proof",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_return_locked_bucket() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 1)
        .take_all_from_worktop(resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "BucketProof",
                "return_bucket_while_locked",
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_pass_locked_bucket_into_method_call_and_back() {
    // This method demonstrates the surprising number of things you can do with a locked bucket
    // in the engine, whilst there is an outstanding proof against the bucket.
    //
    // At validation time:
    // * The transaction validator prevents this for user transactions (but doesn't run for test transactions)
    // * We have to use `build_no_validate()` in the manifest builder to avoid its auto-validation on build
    //
    // At runtime, the main protection mechanisms are currently three-fold:
    // * An error when passing non-global-non-direct-references via argument payloads, meaning the
    //   only way to pass references is on substates of owned objects.
    //   See `test_send_and_receive_reference_from_child_call_frame` in `references.rs`
    // * Checks on references in substates to prevent dropping a bucket with references (e.g. proofs) against it:
    //   - Only transient nodes can store in their substates (See the tests in references.rs)
    //     See `test_send_and_receive_reference_wrapped_in_non_transient_wrapper` in `references.rs`
    //   - non_global_node_refs - The kernel keeps track of the reference count of outstanding references
    //     in open substates. By the previous point, only transient nodes can have references, so this is
    //     a complete list of references. Then the kernel prevents dropping a node if it has outstanding
    //     references.
    // * Application-specific logic in the bucket / proof to prevent withdrawing non-liquid balance.
    //
    // Note that this test depends on the fact that the worktop returns a bucket as-is if there's
    // only one bucket. So, despite appearances, in the below, bucket2 IS bucket1 which IS the bucket
    // orginally created in the account, and all the proofs are against this same bucket.
    //
    // This test breaks if any of these things change:
    // * We withdraw a 0.5 and a 0.5 instead of withdrawing a 1
    // * The method "check_balance_and_bounce" changed to move the balance to a new bucket
    // * We call "split_bucket" at any point to partially separate the bucket

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let builder = ManifestBuilder::new();
    let lookup = builder.name_lookup();
    let manifest = builder
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, dec!(1))
        .take_all_from_worktop(resource_address, "bucket1")
        .create_proof_from_bucket_of_all("bucket1", "proof1_for_holding")
        .create_proof_from_bucket_of_all("bucket1", "proof1_for_auth_zone")
        .push_to_auth_zone("proof1_for_auth_zone")
        .call_function(
            package_address,
            "BucketProof",
            "check_balance_and_bounce",
            (lookup.bucket("bucket1"), dec!(1)),
        )
        .take_all_from_worktop(resource_address, "bucket2")
        .create_proof_from_bucket_of_all("bucket2", "proof2_for_holding")
        .create_proof_from_bucket_of_all("bucket2", "proof2_for_auth_zone")
        .push_to_auth_zone("proof2_for_auth_zone")
        .call_function(
            package_address,
            "BucketProof",
            "check_balance_and_bounce",
            (lookup.bucket("bucket2"), dec!(1)),
        )
        // Now we check the proofs
        .call_function(
            package_address,
            "BucketProof",
            "check_proof_amount_and_drop",
            (lookup.proof("proof1_for_holding"), dec!(1)),
        )
        .call_function(
            package_address,
            "BucketProof",
            "check_proof_amount_and_drop",
            (lookup.proof("proof2_for_holding"), dec!(1)),
        )
        // Create composite proof -- note this proof is just of the single bucket (going by two names), so has a total of 1
        .create_proof_from_auth_zone_of_all(resource_address, "proof_total")
        .call_function(
            package_address,
            "BucketProof",
            "check_proof_amount_and_drop",
            (lookup.proof("proof_total"), dec!(1)),
        )
        .drop_auth_zone_proofs()
        .try_deposit_entire_worktop_or_abort(account, None)
        .build_no_validate();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof_by_amount() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account(account, resource_address, 1)
                .take_all_from_worktop(resource_address, "bucket")
                .with_name_lookup(|builder, lookup| {
                    builder.call_function(
                        package_address,
                        "VaultProof",
                        "new",
                        manifest_args!(lookup.bucket("bucket")),
                    )
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 99)
        .take_from_worktop(resource_address, 99, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof_by_amount",
                manifest_args!(lookup.bucket("bucket"), Decimal::from(2u32)),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof_by_ids() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_non_fungible_resource(account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_non_fungibles_from_account(
                    account,
                    resource_address,
                    [NonFungibleLocalId::integer(1)],
                )
                .take_all_from_worktop(resource_address, "bucket")
                .with_name_lookup(|builder, lookup| {
                    builder.call_function(
                        package_address,
                        "VaultProof",
                        "new",
                        manifest_args!(lookup.bucket("bucket")),
                    )
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(
            account,
            resource_address,
            [
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ],
        )
        .take_non_fungibles_from_worktop(
            resource_address,
            [
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ],
            "bucket",
        )
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof_by_ids",
                manifest_args!(
                    lookup.bucket("bucket"),
                    indexset!(
                        NonFungibleLocalId::integer(1),
                        NonFungibleLocalId::integer(2),
                    )
                ),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_auth_zone_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_non_fungible_resource(account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ],
        )
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            [NonFungibleLocalId::integer(3)],
        )
        .create_proof_from_auth_zone_of_non_fungibles(
            resource_address,
            [
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ],
            "proof",
        )
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "Receiver",
                "assert_ids",
                manifest_args!(
                    lookup.proof("proof"),
                    [
                        NonFungibleLocalId::integer(2),
                        NonFungibleLocalId::integer(3)
                    ],
                    resource_address
                ),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_not_call_vault_lock_fungible_amount_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(btreeset![], |builder| {
        builder.call_function(
            package_address,
            "VaultLockUnlockAuth",
            "new_fungible",
            manifest_args!(),
        )
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "call_lock_fungible_amount_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_vault_unlock_fungible_amount_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(btreeset![], |builder| {
        builder.call_function(
            package_address,
            "VaultLockUnlockAuth",
            "new_fungible",
            manifest_args!(),
        )
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "call_lock_fungible_amount_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_vault_lock_non_fungibles_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(btreeset![], |builder| {
        builder.call_function(
            package_address,
            "VaultLockUnlockAuth",
            "new_non_fungible",
            manifest_args!(),
        )
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "call_lock_non_fungibles_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_vault_unlock_non_fungibles_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));
    let component_address = ledger.new_component(btreeset![], |builder| {
        builder.call_function(
            package_address,
            "VaultLockUnlockAuth",
            "new_non_fungible",
            manifest_args!(),
        )
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "call_lock_non_fungibles_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_bucket_lock_fungible_amount_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "BucketLockUnlockAuth",
            "call_lock_fungible_amount_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_bucket_unlock_fungible_amount_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "BucketLockUnlockAuth",
            "call_lock_fungible_amount_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_bucket_lock_non_fungibles_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "BucketLockUnlockAuth",
            "call_lock_non_fungibles_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn can_not_call_bucket_unlock_non_fungibles_directly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "BucketLockUnlockAuth",
            "call_lock_non_fungibles_directly",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}

#[test]
fn test_proof_check() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_fungible_resource(dec!(100), 0, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resource_address, 1)
        .create_proof_from_auth_zone_of_amount(resource_address, dec!(1), "proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "Receiver",
                "check_if_xrd",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(e)) if e.eq("Invalid proof: Expected ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6), but got ResourceAddress(5dbd2333630248b3e688c93892cec2d199bd917b8a4e019864a552e1f774)") => true,
        _ => false,
    });
}

#[test]
fn test_proof_check_with_message() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_fungible_resource(dec!(100), 0, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resource_address, 1)
        .create_proof_from_auth_zone_of_amount(resource_address, dec!(1), "proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "Receiver",
                "check_with_message_if_xrd",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(e))
            if e.eq("Not XRD proof") =>
        {
            true
        }
        _ => false,
    });
}
