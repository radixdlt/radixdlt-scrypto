use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::system::system_modules::node_move::NodeMoveError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto::resource::DIVISIBILITY_MAXIMUM;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn can_create_clone_and_drop_bucket_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
                manifest_args!(lookup.bucket("bucket"), dec!("1")),
            )
        })
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
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
fn can_create_clone_and_drop_vault_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account(account, resource_address, 1)
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
            "create_clone_drop_vault_proof",
            manifest_args!(Decimal::one()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
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
            manifest_args!(dec!("3"), dec!("1")),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_bucket_for_authorization() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_vault_for_authorization() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
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
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_proof_from_account_and_pass_on() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cant_move_restricted_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::NodeMoveError(
                NodeMoveError::CantMoveDownstream(..)
            ))
        )
    });
}

#[test]
fn can_move_restricted_proofs_internally() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let (public_key, _, account) = test_runner.new_allocated_account();
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "Outer", "instantiate", manifest_args!())
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
        receipt.expect_commit_success().new_component_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!("1"))
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "pass_fungible_proof",
                manifest_args!(lookup.proof("proof")),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_move_locked_bucket() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
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
                "compose_vault_and_bucket_proof",
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof_by_amount() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
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
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_compose_bucket_and_vault_proof_by_ids() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_non_fungibles_from_account(
                    account,
                    resource_address,
                    &btreeset!(NonFungibleLocalId::integer(1)),
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
            &BTreeSet::from([
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]),
        )
        .take_non_fungibles_from_worktop(
            resource_address,
            &BTreeSet::from([
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]),
            "bucket",
        )
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof_by_ids",
                manifest_args!(
                    lookup.bucket("bucket"),
                    BTreeSet::from([
                        NonFungibleLocalId::integer(1),
                        NonFungibleLocalId::integer(2),
                    ])
                ),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_vault_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(
        btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account(account, resource_address, 3)
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
        .call_method(
            component_address,
            "create_clone_drop_vault_proof_by_amount",
            manifest_args!(Decimal::from(3), Decimal::from(1)),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_create_auth_zone_proof_by_amount_from_non_fungibles() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            &BTreeSet::from([
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ]),
        )
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            &BTreeSet::from([NonFungibleLocalId::integer(3)]),
        )
        .create_proof_from_auth_zone_of_non_fungibles(
            resource_address,
            &BTreeSet::from([
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]),
            "proof",
        )
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "Receiver",
                "assert_ids",
                manifest_args!(
                    lookup.proof("proof"),
                    BTreeSet::from([
                        NonFungibleLocalId::integer(2),
                        NonFungibleLocalId::integer(3)
                    ]),
                    resource_address
                ),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_not_call_vault_lock_fungible_amount_directly() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(btreeset![], |builder| {
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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(btreeset![], |builder| {
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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(btreeset![], |builder| {
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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.new_component(btreeset![], |builder| {
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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            _,
        ))) => true,
        _ => false,
    })
}
