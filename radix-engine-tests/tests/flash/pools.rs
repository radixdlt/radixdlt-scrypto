use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::blueprints::pool::v1::errors::{
    multi_resource_pool::Error as MultiResourcePoolError,
    two_resource_pool::Error as TwoResourcePoolError,
};
use radix_engine::updates::*;
use scrypto_test::prelude::*;

#[test]
fn database_is_consistent_before_and_after_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.only_babylon())
        .build();

    let (pk, _, account) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&pk);

    let fungible1 = ledger.create_fungible_resource(dec!(200), 18, account);
    let fungible2 = ledger.create_fungible_resource(dec!(200), 18, account);

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    POOL_PACKAGE,
                    ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    OneResourcePoolInstantiateManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                        resource_address: fungible1.into(),
                        address_reservation: None,
                    },
                )
                .call_function(
                    POOL_PACKAGE,
                    TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                    TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                    TwoResourcePoolInstantiateManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                        resource_addresses: (fungible1.into(), fungible2.into()),
                        address_reservation: None,
                    },
                )
                .call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                    MultiResourcePoolInstantiateManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                        resource_addresses: indexset! {fungible1.into(), fungible2.into()},
                        address_reservation: None,
                    },
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![],
        )
        .expect_commit_success();
    ledger.check_database();

    // Act
    {
        ProtocolUpdateExecutor::new(
            NetworkDefinition::simulator(),
            AnemoneSettings::all_disabled().enable(|item| &mut item.pools_update),
        )
        .run_and_commit(ledger.substate_db_mut());
    }

    // Assert
    ledger.check_database();
}

#[test]
fn single_sided_contributions_to_two_resource_pool_are_only_allowed_after_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.only_babylon())
        .build();

    let (pk, _, account) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&pk);

    let fungible1 = ledger.create_fungible_resource(dec!(200), 18, account);
    let fungible2 = ledger.create_fungible_resource(dec!(200), 18, account);

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, fungible1, dec!(100))
            .withdraw_from_account(account, fungible2, dec!(100))
            .take_all_from_worktop(fungible1, "bucket1")
            .take_all_from_worktop(fungible2, "bucket2")
            .allocate_global_address(
                POOL_PACKAGE,
                TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                "reservation",
                "address_name",
            )
            .with_name_lookup(|builder, _| {
                let reservation = builder.address_reservation("reservation");
                let named_address = builder.named_address("address_name");

                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder
                    .call_function(
                        POOL_PACKAGE,
                        TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                        TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                        TwoResourcePoolInstantiateManifestInput {
                            owner_role: OwnerRole::None,
                            pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                            resource_addresses: (fungible1.into(), fungible2.into()),
                            address_reservation: Some(reservation),
                        },
                    )
                    .call_method(
                        named_address,
                        TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                        TwoResourcePoolContributeManifestInput {
                            buckets: (bucket1, bucket2),
                        },
                    )
                    .call_method(
                        named_address,
                        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        TwoResourcePoolProtectedWithdrawManifestInput {
                            resource_address: fungible1.into(),
                            amount: dec!(100),
                            withdraw_strategy: WithdrawStrategy::Exact,
                        },
                    )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge.clone()],
    );

    let pool_address = receipt
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();
    let pool_unit = receipt
        .expect_commit_success()
        .new_resource_addresses()
        .first()
        .copied()
        .unwrap();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, fungible1, dec!(100))
            .withdraw_from_account(account, fungible2, dec!(100))
            .take_all_from_worktop(fungible1, "bucket1")
            .take_all_from_worktop(fungible2, "bucket2")
            .with_name_lookup(|builder, _| {
                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder.call_method(
                    pool_address,
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    TwoResourcePoolContributeManifestInput {
                        buckets: (bucket1, bucket2),
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge.clone()],
    );

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
                TwoResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves
            ))
        )
    });

    // Act
    {
        ProtocolUpdateExecutor::new(
            NetworkDefinition::simulator(),
            AnemoneSettings::all_disabled().enable(|item| &mut item.pools_update),
        )
        .run_and_commit(ledger.substate_db_mut());
    }
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, fungible1, dec!(100))
            .withdraw_from_account(account, fungible2, dec!(100))
            .take_all_from_worktop(fungible1, "bucket1")
            .take_all_from_worktop(fungible2, "bucket2")
            .with_name_lookup(|builder, _| {
                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder.call_method(
                    pool_address,
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    TwoResourcePoolContributeManifestInput {
                        buckets: (bucket1, bucket2),
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.get_component_balance(account, fungible1), dec!(200));
    assert_eq!(ledger.get_component_balance(account, fungible2), dec!(0));
    assert_eq!(ledger.get_component_balance(account, pool_unit), dec!(200));
}

#[test]
fn single_sided_contributions_to_multi_resource_pool_are_only_allowed_after_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.only_babylon())
        .build();

    let (pk, _, account) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&pk);

    let fungible1 = ledger.create_fungible_resource(dec!(200), 18, account);
    let fungible2 = ledger.create_fungible_resource(dec!(200), 18, account);

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, fungible1, dec!(100))
            .withdraw_from_account(account, fungible2, dec!(100))
            .take_all_from_worktop(fungible1, "bucket1")
            .take_all_from_worktop(fungible2, "bucket2")
            .allocate_global_address(
                POOL_PACKAGE,
                MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                "reservation",
                "address_name",
            )
            .with_name_lookup(|builder, _| {
                let reservation = builder.address_reservation("reservation");
                let named_address = builder.named_address("address_name");

                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder
                    .call_function(
                        POOL_PACKAGE,
                        MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                        MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                        MultiResourcePoolInstantiateManifestInput {
                            owner_role: OwnerRole::None,
                            pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                            resource_addresses: indexset![fungible1.into(), fungible2.into()],
                            address_reservation: Some(reservation),
                        },
                    )
                    .call_method(
                        named_address,
                        MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                        MultiResourcePoolContributeManifestInput {
                            buckets: ManifestBucketBatch::ManifestBuckets(vec![bucket1, bucket2]),
                        },
                    )
                    .call_method(
                        named_address,
                        MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        MultiResourcePoolProtectedWithdrawManifestInput {
                            resource_address: fungible1.into(),
                            amount: dec!(100),
                            withdraw_strategy: WithdrawStrategy::Exact,
                        },
                    )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge.clone()],
    );

    let pool_address = receipt
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();
    let pool_unit = receipt
        .expect_commit_success()
        .new_resource_addresses()
        .first()
        .copied()
        .unwrap();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, fungible1, dec!(100))
            .withdraw_from_account(account, fungible2, dec!(100))
            .take_all_from_worktop(fungible1, "bucket1")
            .take_all_from_worktop(fungible2, "bucket2")
            .with_name_lookup(|builder, _| {
                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder.call_method(
                    pool_address,
                    MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    MultiResourcePoolContributeManifestInput {
                        buckets: ManifestBucketBatch::ManifestBuckets(vec![bucket1, bucket2]),
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge.clone()],
    );

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
                MultiResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves
            ))
        )
    });

    // Act
    {
        ProtocolUpdateExecutor::new(
            NetworkDefinition::simulator(),
            AnemoneSettings::all_disabled().enable(|item| &mut item.pools_update),
        )
        .run_and_commit(ledger.substate_db_mut());
    }
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, fungible1, dec!(100))
            .withdraw_from_account(account, fungible2, dec!(100))
            .take_all_from_worktop(fungible1, "bucket1")
            .take_all_from_worktop(fungible2, "bucket2")
            .with_name_lookup(|builder, _| {
                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder.call_method(
                    pool_address,
                    MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    MultiResourcePoolContributeManifestInput {
                        buckets: ManifestBucketBatch::ManifestBuckets(vec![bucket1, bucket2]),
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.get_component_balance(account, fungible1), dec!(200));
    assert_eq!(ledger.get_component_balance(account, fungible2), dec!(0));
    assert_eq!(ledger.get_component_balance(account, pool_unit), dec!(200));
}
