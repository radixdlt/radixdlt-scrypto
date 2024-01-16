use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::blueprints::pool::v1::errors::{
    multi_resource_pool::Error as MultiResourcePoolError,
    two_resource_pool::Error as TwoResourcePoolError,
};
use radix_engine::system::system_db_reader::*;
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine_queries::typed_substate_layout::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;

#[test]
fn database_is_consistent_before_and_after_protocol_update() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .without_pools_v1_1()
        .without_trace()
        .build();

    let (pk, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&pk);

    let fungible1 = test_runner.create_fungible_resource(dec!(200), 18, account);
    let fungible2 = test_runner.create_fungible_resource(dec!(200), 18, account);

    let new_components = test_runner
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
                        resource_address: fungible1,
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
                        resource_addresses: (fungible1, fungible2),
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
                        resource_addresses: indexset! {fungible1, fungible2},
                        address_reservation: None,
                    },
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .clone();
    test_runner.check_database();

    // Act
    {
        let substate_db = test_runner.substate_db_mut();
        let state_updates = pools_package_v1_1::generate_state_updates(substate_db);
        let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
        substate_db.commit(&db_updates);
    }

    // Assert
    test_runner.check_database();

    let reader = SystemDatabaseReader::new(test_runner.substate_db());
    for pool_address in new_components.iter() {
        let pool_manager_role = reader
            .read_object_collection_entry::<_, RoleAssignmentAccessRuleEntryPayload>(
                pool_address.as_node_id(),
                ModuleId::RoleAssignment,
                ObjectCollectionKey::KeyValue(
                    RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
                    &ModuleRoleKey::new(ModuleId::Main, POOL_MANAGER_ROLE),
                ),
            )
            .unwrap()
            .unwrap()
            .into_latest();
        let pool_contributor_role = reader
            .read_object_collection_entry::<_, RoleAssignmentAccessRuleEntryPayload>(
                pool_address.as_node_id(),
                ModuleId::RoleAssignment,
                ObjectCollectionKey::KeyValue(
                    RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
                    &ModuleRoleKey::new(ModuleId::Main, POOL_CONTRIBUTOR_ROLE),
                ),
            )
            .unwrap()
            .unwrap()
            .into_latest();

        assert_eq!(pool_manager_role, pool_contributor_role);
        assert_eq!(
            pool_manager_role,
            rule!(require(virtual_signature_badge.clone()))
        );
    }
}

#[test]
fn single_sided_contributions_to_two_resource_pool_are_only_allowed_after_protocol_update() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .without_pools_v1_1()
        .without_trace()
        .build();

    let (pk, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&pk);

    let fungible1 = test_runner.create_fungible_resource(dec!(200), 18, account);
    let fungible2 = test_runner.create_fungible_resource(dec!(200), 18, account);

    let receipt = test_runner.execute_manifest(
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
                            resource_addresses: (fungible1, fungible2),
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
                            resource_address: fungible1,
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
    let receipt = test_runner.execute_manifest(
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
        let substate_db = test_runner.substate_db_mut();
        let state_updates = pools_package_v1_1::generate_state_updates(substate_db);
        let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
        substate_db.commit(&db_updates);
    }
    let receipt = test_runner.execute_manifest(
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
    assert_eq!(
        test_runner.get_component_balance(account, fungible1),
        dec!(200)
    );
    assert_eq!(
        test_runner.get_component_balance(account, fungible2),
        dec!(0)
    );
    assert_eq!(
        test_runner.get_component_balance(account, pool_unit),
        dec!(200)
    );
}

#[test]
fn single_sided_contributions_to_multi_resource_pool_are_only_allowed_after_protocol_update() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .without_pools_v1_1()
        .without_trace()
        .build();

    let (pk, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&pk);

    let fungible1 = test_runner.create_fungible_resource(dec!(200), 18, account);
    let fungible2 = test_runner.create_fungible_resource(dec!(200), 18, account);

    let receipt = test_runner.execute_manifest(
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
                            resource_addresses: indexset![fungible1, fungible2],
                            address_reservation: Some(reservation),
                        },
                    )
                    .call_method(
                        named_address,
                        MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                        MultiResourcePoolContributeManifestInput {
                            buckets: vec![bucket1, bucket2],
                        },
                    )
                    .call_method(
                        named_address,
                        MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        MultiResourcePoolProtectedWithdrawManifestInput {
                            resource_address: fungible1,
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
    let receipt = test_runner.execute_manifest(
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
                        buckets: vec![bucket1, bucket2],
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
        let substate_db = test_runner.substate_db_mut();
        let state_updates = pools_package_v1_1::generate_state_updates(substate_db);
        let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
        substate_db.commit(&db_updates);
    }
    let receipt = test_runner.execute_manifest(
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
                        buckets: vec![bucket1, bucket2],
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![virtual_signature_badge],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.get_component_balance(account, fungible1),
        dec!(200)
    );
    assert_eq!(
        test_runner.get_component_balance(account, fungible2),
        dec!(0)
    );
    assert_eq!(
        test_runner.get_component_balance(account, pool_unit),
        dec!(200)
    );
}

#[test]
fn contributing_to_one_resource_pool_without_any_badge_fails() {
    test_contribution_with_pools_created_in_v1_1(PoolType::One, ContributeWithMinor1::NoBadge)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_one_resource_pool_with_a_manager_badge_fails() {
    test_contribution_with_pools_created_in_v1_1(PoolType::One, ContributeWithMinor1::ManagerRole)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_one_resource_pool_with_a_contributor_badge_succeeds() {
    test_contribution_with_pools_created_in_v1_1(
        PoolType::One,
        ContributeWithMinor1::ContributorRole,
    )
    .expect_commit_success();
}

#[test]
fn contributing_to_two_resource_pool_without_any_badge_fails() {
    test_contribution_with_pools_created_in_v1_1(PoolType::Two, ContributeWithMinor1::NoBadge)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_two_resource_pool_with_a_manager_badge_fails() {
    test_contribution_with_pools_created_in_v1_1(PoolType::Two, ContributeWithMinor1::ManagerRole)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_two_resource_pool_with_a_contributor_badge_succeeds() {
    test_contribution_with_pools_created_in_v1_1(
        PoolType::Two,
        ContributeWithMinor1::ContributorRole,
    )
    .expect_commit_success();
}

#[test]
fn contributing_to_multi_resource_pool_without_any_badge_fails() {
    test_contribution_with_pools_created_in_v1_1(PoolType::Multi, ContributeWithMinor1::NoBadge)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_multi_resource_pool_with_a_manager_badge_fails() {
    test_contribution_with_pools_created_in_v1_1(
        PoolType::Multi,
        ContributeWithMinor1::ManagerRole,
    )
    .expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized(error)
            )) if error.fn_identifier.ident == "contribute"
        )
    });
}

#[test]
fn contributing_to_multi_resource_pool_with_a_contributor_badge_succeeds() {
    test_contribution_with_pools_created_in_v1_1(
        PoolType::Multi,
        ContributeWithMinor1::ContributorRole,
    )
    .expect_commit_success();
}

#[test]
fn contributing_to_an_upgraded_one_resource_pool_without_any_badge_fails() {
    test_contribution_with_pools_upgraded_to_v1_1(PoolType::One, ContributeWithMinor0::NoBadge)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_an_upgraded_one_resource_pool_with_a_manager_badge_succeeds() {
    test_contribution_with_pools_upgraded_to_v1_1(PoolType::One, ContributeWithMinor0::ManagerRole)
        .expect_commit_success();
}

#[test]
fn contributing_to_an_upgraded_two_resource_pool_without_any_badge_fails() {
    test_contribution_with_pools_upgraded_to_v1_1(PoolType::Two, ContributeWithMinor0::NoBadge)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_an_upgraded_two_resource_pool_with_a_manager_badge_succeeds() {
    test_contribution_with_pools_upgraded_to_v1_1(PoolType::Two, ContributeWithMinor0::ManagerRole)
        .expect_commit_success();
}

#[test]
fn contributing_to_an_upgraded_multi_resource_pool_without_any_badge_fails() {
    test_contribution_with_pools_upgraded_to_v1_1(PoolType::Multi, ContributeWithMinor0::NoBadge)
        .expect_specific_failure(|runtime_error| {
            matches!(
                runtime_error,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                    AuthError::Unauthorized(error)
                )) if error.fn_identifier.ident == "contribute"
            )
        });
}

#[test]
fn contributing_to_an_upgraded_multi_resource_pool_with_a_manager_badge_succeeds() {
    test_contribution_with_pools_upgraded_to_v1_1(
        PoolType::Multi,
        ContributeWithMinor0::ManagerRole,
    )
    .expect_commit_success();
}

/// This tests that it it's not valid for the v1.0 package and all tests that use the
/// `test_contribution_with_pools_created_in_v1_1` function test that it is valid for v1.1.
#[test]
fn instantiate_with_contributor_rule_is_not_available_for_v1_0_one_resource_pool() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .without_trace()
        .without_pools_v1_1()
        .build();
    let (_, _, account) = test_runner.new_account(false);
    let fungible1 = test_runner.create_fungible_resource(dec!(0), 18, account);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            POOL_PACKAGE,
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
            ONE_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT,
            OneResourcePoolInstantiateWithContributorRuleManifestInput {
                owner_role: OwnerRole::None,
                pool_manager_rule: rule!(allow_all),
                pool_contributor_rule: rule!(allow_all),
                resource_address: fungible1,
                address_reservation: None,
            },
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadDoesNotExist(_, BlueprintPayloadIdentifier::Function(fn_ident, _))
            )) if fn_ident == ONE_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT
        )
    });
}

/// This tests that it it's not valid for the v1.0 package and all tests that use the
/// `test_contribution_with_pools_created_in_v1_1` function test that it is valid for v1.1.
#[test]
fn instantiate_with_contributor_rule_is_not_available_for_v1_0_two_resource_pool() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .without_trace()
        .without_pools_v1_1()
        .build();
    let (_, _, account) = test_runner.new_account(false);
    let fungible1 = test_runner.create_fungible_resource(dec!(0), 18, account);
    let fungible2 = test_runner.create_fungible_resource(dec!(0), 18, account);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            POOL_PACKAGE,
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
            TWO_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT,
            TwoResourcePoolInstantiateWithContributorRuleManifestInput {
                owner_role: OwnerRole::None,
                pool_manager_rule: rule!(allow_all),
                pool_contributor_rule: rule!(allow_all),
                resource_addresses: (fungible1, fungible2),
                address_reservation: None,
            },
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadDoesNotExist(_, BlueprintPayloadIdentifier::Function(fn_ident, _))
            )) if fn_ident == TWO_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT
        )
    });
}

/// This tests that it it's not valid for the v1.0 package and all tests that use the
/// `test_contribution_with_pools_created_in_v1_1` function test that it is valid for v1.1.
#[test]
fn instantiate_with_contributor_rule_is_not_available_for_v1_0_multi_resource_pool() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .without_trace()
        .without_pools_v1_1()
        .build();
    let (_, _, account) = test_runner.new_account(false);
    let fungible1 = test_runner.create_fungible_resource(dec!(0), 18, account);
    let fungible2 = test_runner.create_fungible_resource(dec!(0), 18, account);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            POOL_PACKAGE,
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
            MULTI_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT,
            MultiResourcePoolInstantiateWithContributorRuleManifestInput {
                owner_role: OwnerRole::None,
                pool_manager_rule: rule!(allow_all),
                pool_contributor_rule: rule!(allow_all),
                resource_addresses: indexset! { fungible1, fungible2 },
                address_reservation: None,
            },
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadDoesNotExist(_, BlueprintPayloadIdentifier::Function(fn_ident, _))
            )) if fn_ident == MULTI_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT
        )
    });
}

/// The pools created by this function are created after the package has been upgraded to v1.1.
/// So, none of the _components_ have been upgraded, just their package and then they were created.
/// This is an important distinction to make since the pool's protocol update also requires changes
/// to the state of components, specifically adding a new role to their role assignment module.
fn test_contribution_with_pools_created_in_v1_1(
    pool_type: PoolType,
    contribute_with: ContributeWithMinor1,
) -> TransactionReceiptV1 {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();

    let (_, _, account) = test_runner.new_account(false);

    let fungible1 =
        test_runner.create_freely_mintable_fungible_resource(OwnerRole::None, None, 18, account);
    let fungible2 =
        test_runner.create_freely_mintable_fungible_resource(OwnerRole::None, None, 18, account);

    let pool_manager_virtual_signature_badge =
        NonFungibleGlobalId::from_public_key(&test_runner.new_key_pair().0);
    let pool_contributor_virtual_signature_badge =
        NonFungibleGlobalId::from_public_key(&test_runner.new_key_pair().0);

    let manifest = {
        let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();

        match pool_type {
            PoolType::One => {
                manifest_builder = manifest_builder.call_function(
                    POOL_PACKAGE,
                    ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    ONE_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT,
                    OneResourcePoolInstantiateWithContributorRuleManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(
                            pool_manager_virtual_signature_badge.clone()
                        )),
                        pool_contributor_rule: rule!(require(
                            pool_contributor_virtual_signature_badge.clone()
                        )),
                        resource_address: fungible1,
                        address_reservation: None,
                    },
                );
            }
            PoolType::Two => {
                manifest_builder = manifest_builder.call_function(
                    POOL_PACKAGE,
                    TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                    TWO_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT,
                    TwoResourcePoolInstantiateWithContributorRuleManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(
                            pool_manager_virtual_signature_badge.clone()
                        )),
                        pool_contributor_rule: rule!(require(
                            pool_contributor_virtual_signature_badge.clone()
                        )),
                        resource_addresses: (fungible1, fungible2),
                        address_reservation: None,
                    },
                );
            }
            PoolType::Multi => {
                manifest_builder = manifest_builder.call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_WITH_CONTRIBUTOR_RULE_IDENT,
                    MultiResourcePoolInstantiateWithContributorRuleManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(
                            pool_manager_virtual_signature_badge.clone()
                        )),
                        pool_contributor_rule: rule!(require(
                            pool_contributor_virtual_signature_badge.clone()
                        )),
                        resource_addresses: indexset! {fungible1, fungible2},
                        address_reservation: None,
                    },
                );
            }
        }

        manifest_builder.build()
    };
    let pool = test_runner
        .execute_manifest(manifest, vec![])
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(fungible1, dec!(100))
        .mint_fungible(fungible2, dec!(100))
        .take_all_from_worktop(fungible1, "bucket1")
        .take_all_from_worktop(fungible2, "bucket2")
        .with_name_lookup(|builder, _| {
            let bucket1 = builder.bucket("bucket1");
            let bucket2 = builder.bucket("bucket2");

            match pool_type {
                PoolType::One => builder.return_to_worktop("bucket2").call_method(
                    pool,
                    ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    OneResourcePoolContributeManifestInput { bucket: bucket1 },
                ),
                PoolType::Two => builder.call_method(
                    pool,
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    TwoResourcePoolContributeManifestInput {
                        buckets: (bucket1, bucket2),
                    },
                ),
                PoolType::Multi => builder.call_method(
                    pool,
                    MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    MultiResourcePoolContributeManifestInput {
                        buckets: vec![bucket1, bucket2],
                    },
                ),
            }
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    let badges = match contribute_with {
        ContributeWithMinor1::NoBadge => vec![],
        ContributeWithMinor1::ManagerRole => vec![pool_manager_virtual_signature_badge],
        ContributeWithMinor1::ContributorRole => vec![pool_contributor_virtual_signature_badge],
    };

    test_runner.execute_manifest(manifest, badges)
}

/// The pools created by this function are first created before the protocol update and then
/// upgraded to v1.1 with the update.
fn test_contribution_with_pools_upgraded_to_v1_1(
    pool_type: PoolType,
    contribute_with: ContributeWithMinor0,
) -> TransactionReceiptV1 {
    let mut test_runner = TestRunnerBuilder::new()
        .without_trace()
        .without_pools_v1_1()
        .build();

    let (_, _, account) = test_runner.new_account(false);

    let fungible1 =
        test_runner.create_freely_mintable_fungible_resource(OwnerRole::None, None, 18, account);
    let fungible2 =
        test_runner.create_freely_mintable_fungible_resource(OwnerRole::None, None, 18, account);

    let pool_manager_virtual_signature_badge =
        NonFungibleGlobalId::from_public_key(&test_runner.new_key_pair().0);

    let manifest = {
        let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();

        match pool_type {
            PoolType::One => {
                manifest_builder = manifest_builder.call_function(
                    POOL_PACKAGE,
                    ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    OneResourcePoolInstantiateManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(
                            pool_manager_virtual_signature_badge.clone()
                        )),
                        resource_address: fungible1,
                        address_reservation: None,
                    },
                );
            }
            PoolType::Two => {
                manifest_builder = manifest_builder.call_function(
                    POOL_PACKAGE,
                    TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                    TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                    TwoResourcePoolInstantiateManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(
                            pool_manager_virtual_signature_badge.clone()
                        )),
                        resource_addresses: (fungible1, fungible2),
                        address_reservation: None,
                    },
                );
            }
            PoolType::Multi => {
                manifest_builder = manifest_builder.call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                    MultiResourcePoolInstantiateManifestInput {
                        owner_role: OwnerRole::None,
                        pool_manager_rule: rule!(require(
                            pool_manager_virtual_signature_badge.clone()
                        )),
                        resource_addresses: indexset! {fungible1, fungible2},
                        address_reservation: None,
                    },
                );
            }
        }

        manifest_builder.build()
    };
    let pool = test_runner
        .execute_manifest(manifest, vec![])
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    let substate_db = test_runner.substate_db_mut();
    let state_updates = pools_package_v1_1::generate_state_updates(substate_db);
    let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
    substate_db.commit(&db_updates);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(fungible1, dec!(100))
        .mint_fungible(fungible2, dec!(100))
        .take_all_from_worktop(fungible1, "bucket1")
        .take_all_from_worktop(fungible2, "bucket2")
        .with_name_lookup(|builder, _| {
            let bucket1 = builder.bucket("bucket1");
            let bucket2 = builder.bucket("bucket2");

            match pool_type {
                PoolType::One => builder.return_to_worktop("bucket2").call_method(
                    pool,
                    ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    OneResourcePoolContributeManifestInput { bucket: bucket1 },
                ),
                PoolType::Two => builder.call_method(
                    pool,
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    TwoResourcePoolContributeManifestInput {
                        buckets: (bucket1, bucket2),
                    },
                ),
                PoolType::Multi => builder.call_method(
                    pool,
                    MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    MultiResourcePoolContributeManifestInput {
                        buckets: vec![bucket1, bucket2],
                    },
                ),
            }
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    let badges = match contribute_with {
        ContributeWithMinor0::NoBadge => vec![],
        ContributeWithMinor0::ManagerRole => vec![pool_manager_virtual_signature_badge],
    };

    test_runner.execute_manifest(manifest, badges)
}

enum PoolType {
    One,
    Two,
    Multi,
}

enum ContributeWithMinor0 {
    NoBadge,
    ManagerRole,
}

enum ContributeWithMinor1 {
    NoBadge,
    ManagerRole,
    ContributorRole,
}
