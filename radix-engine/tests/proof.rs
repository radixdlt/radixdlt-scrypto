use radix_engine::engine::node_move_module::NodeMoveError;
use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::model::FromPublicKey;
use scrypto::resource::DIVISIBILITY_MAXIMUM;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use utils::ContextualDisplay;

#[test]
fn can_create_clone_and_drop_bucket_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .withdraw_from_account_by_amount(account, 1.into(), resource_address)
        .take_from_worktop(resource_address, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "BucketProof",
                "create_clone_drop_bucket_proof",
                args!(bucket_id, dec!("1")),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 1.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(
            component_address,
            "create_clone_drop_vault_proof",
            args!(Decimal::one()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 3.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(
            component_address,
            "create_clone_drop_vault_proof_by_amount",
            args!(dec!("3"), dec!("1")),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 3.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let total_ids = BTreeSet::from([
        NonFungibleLocalId::integer(1),
        NonFungibleLocalId::integer(2),
        NonFungibleLocalId::integer(3),
    ]);
    let proof_ids = BTreeSet::from([NonFungibleLocalId::integer(2)]);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (auth_resource_address, burnable_resource_address) =
        test_runner.create_restricted_burn_token(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .withdraw_from_account_by_amount(account, 1.into(), auth_resource_address)
        .withdraw_from_account_by_amount(account, 1.into(), burnable_resource_address)
        .take_from_worktop(auth_resource_address, |builder, auth_bucket_id| {
            builder.take_from_worktop(burnable_resource_address, |builder, burnable_bucket_id| {
                builder.call_function(
                    package_address,
                    "BucketProof",
                    "use_bucket_proof_for_auth",
                    args!(auth_bucket_id, burnable_bucket_id),
                )
            })
        })
        .call_method(
            account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 1.into(), auth_resource_address)
                .take_from_worktop(auth_resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .withdraw_from_account_by_amount(account, 1.into(), burnable_resource_address)
        .take_from_worktop(burnable_resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "use_vault_proof_for_auth",
                args!(bucket_id),
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
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_proof_from_account_by_amount(account, 1.into(), resource_address)
        .pop_from_auth_zone(|builder, proof_id| {
            builder.call_function(
                package_address,
                "VaultProof",
                "receive_proof",
                args!(proof_id),
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
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .create_proof_from_account_by_amount(account, 1.into(), resource_address)
        .pop_from_auth_zone(|builder, proof_id| {
            builder.call_function(
                package_address,
                "VaultProof",
                "receive_proof_and_push_to_auth_zone",
                args!(proof_id),
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
            RuntimeError::ModuleError(ModuleError::NodeMoveError(
                NodeMoveError::CantMoveDownstream(RENodeId::Proof(..))
            ))
        )
    });
}

#[test]
fn cant_move_locked_bucket() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .withdraw_from_account_by_amount(account, 1.into(), resource_address)
        .take_from_worktop(resource_address, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "BucketProof",
                "return_bucket_while_locked",
                args!(bucket_id),
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
            RuntimeError::ModuleError(ModuleError::NodeMoveError(NodeMoveError::CantMoveUpstream(
                RENodeId::Bucket(..)
            )))
        )
    });
}

#[test]
fn can_compose_bucket_and_vault_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100u32.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 1.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .withdraw_from_account_by_amount(account, 99u32.into(), resource_address)
        .take_from_worktop_by_amount(99u32.into(), resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof",
                args!(bucket_id),
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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 1.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .withdraw_from_account_by_amount(account, 99u32.into(), resource_address)
        .take_from_worktop_by_amount(99u32.into(), resource_address, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "compose_vault_and_bucket_proof_by_amount",
                args!(bucket_id, Decimal::from(2u32)),
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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 1.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .withdraw_from_account_by_ids(
            account,
            &BTreeSet::from([
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]),
            resource_address,
        )
        .take_from_worktop_by_ids(
            &BTreeSet::from([
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]),
            resource_address,
            |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "compose_vault_and_bucket_proof_by_ids",
                    args!(
                        bucket_id,
                        BTreeSet::from([
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ])
                    ),
                )
            },
        )
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
    let component_address = test_runner.instantiate_component(
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
        |builder| {
            builder
                .withdraw_from_account_by_amount(account, 3.into(), resource_address)
                .take_from_worktop(resource_address, |builder, bucket_id| {
                    builder.call_function(package_address, "VaultProof", "new", args!(bucket_id))
                })
        },
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .create_proof_from_account_by_ids(
            account,
            &BTreeSet::from([
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ]),
            resource_address,
        )
        .create_proof_from_account_by_ids(
            account,
            &BTreeSet::from([NonFungibleLocalId::integer(3)]),
            resource_address,
        )
        .create_proof_from_auth_zone_by_ids(
            &BTreeSet::from([
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]),
            resource_address,
            |builder, proof_id| {
                builder.call_function(
                    package_address,
                    "Receiver",
                    "assert_ids",
                    args!(
                        proof_id,
                        BTreeSet::from([
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(3)
                        ]),
                        resource_address
                    ),
                )
            },
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
