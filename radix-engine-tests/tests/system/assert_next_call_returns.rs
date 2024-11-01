use radix_engine::blueprints::pool::v1::constants::TWO_RESOURCE_POOL_BLUEPRINT_IDENT;
use scrypto_test::prelude::*;

const NEXT_CALL_TYPES: [NextCallType; 3] = [
    NextCallType::MethodCall,
    NextCallType::YieldToParent,
    NextCallType::YieldToChild,
];

#[test]
fn when_more_is_returned_assert_next_call_returns_include_should_succeed() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, _resource2| {
                ManifestResourceConstraints::new().with(
                    resource1,
                    ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                )
            },
            false,
            next_call_type,
            |_, _| None,
        );
    }
}

#[test]
fn when_more_is_returned_assert_next_call_returns_only_should_fail() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, _resource2| {
                ManifestResourceConstraints::new().with(
                    resource1,
                    ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                )
            },
            true,
            next_call_type,
            |_, resource2| {
                Some(
                    ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                        resource_address: resource2,
                    },
                )
            },
        );
    }
}

#[test]
fn when_exact_is_returned_assert_next_call_returns_include_should_succeed() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, resource2| {
                ManifestResourceConstraints::new()
                    .with(
                        resource1,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(
                        resource2,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
            },
            false,
            next_call_type,
            |_, _| None,
        );
    }
}

#[test]
fn when_exact_is_returned_assert_next_call_returns_only_should_succeed() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, resource2| {
                ManifestResourceConstraints::new()
                    .with(
                        resource1,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(
                        resource2,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
            },
            true,
            next_call_type,
            |_, _| None,
        );
    }
}

#[test]
fn when_less_is_returned_assert_next_call_returns_include_should_fail() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, _resource2| {
                ManifestResourceConstraints::new()
                    .with(
                        resource1,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(XRD, ManifestResourceConstraint::AtLeastAmount(dec!(1)))
            },
            false,
            next_call_type,
            |_, _| {
                Some(ResourceConstraintsError::ResourceConstraintFailed {
                    resource_address: XRD,
                    error: ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount: dec!(1),
                        actual_amount: dec!(0),
                    },
                })
            },
        );
    }
}

#[test]
fn when_less_is_returned_assert_next_call_returns_only_should_fail() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, _resource2| {
                ManifestResourceConstraints::new()
                    .with(
                        resource1,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(XRD, ManifestResourceConstraint::AtLeastAmount(dec!(1)))
            },
            true,
            next_call_type,
            |_, resource2| {
                Some(
                    ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                        resource_address: resource2,
                    },
                )
            },
        );
    }
}

#[test]
fn when_empty_constraints_on_assert_next_call_returns_include_should_succeed() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |_resource1, _resource2| ManifestResourceConstraints::new(),
            false,
            next_call_type,
            |_, _| None,
        );
    }
}

#[test]
fn when_empty_constraints_on_assert_next_call_returns_only_should_fail() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |_resource1, _resource2| ManifestResourceConstraints::new(),
            true,
            next_call_type,
            |resource1, _resource2| {
                Some(
                    ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                        resource_address: resource1,
                    },
                )
            },
        );
    }
}

#[test]
fn when_extra_zero_constraints_on_assert_next_call_returns_include_should_succeed() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, resource2| {
                ManifestResourceConstraints::new()
                    .with(
                        resource1,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(
                        resource2,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(XRD, ManifestResourceConstraint::ExactAmount(dec!(0)))
            },
            false,
            next_call_type,
            |_, _| None,
        );
    }
}

#[test]
fn when_extra_zero_constraints_on_assert_next_call_returns_only_should_succeed() {
    for next_call_type in NEXT_CALL_TYPES {
        run_return_two_resources_test(
            |resource1, resource2| {
                ManifestResourceConstraints::new()
                    .with(
                        resource1,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(
                        resource2,
                        ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                    )
                    .with(XRD, ManifestResourceConstraint::ExactAmount(dec!(0)))
            },
            true,
            next_call_type,
            |_, _| None,
        );
    }
}

#[test]
fn when_withdrawing_zero_amount_with_zero_constraints_on_assert_next_call_returns_only_should_succeed(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_from_faucet()
        .assert_next_call_returns_only(
            ManifestResourceConstraints::new()
                .with(XRD, ManifestResourceConstraint::ExactAmount(dec!(0))),
        )
        .withdraw_from_account(account, XRD, dec!(0))
        .build();
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(manifest, [public_key.signature_proof()]);
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn when_withdrawing_zero_non_fungibles_with_zero_constraints_on_assert_next_call_returns_only_should_succeed(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_from_faucet()
        .assert_next_call_returns_only(ManifestResourceConstraints::new().with(
            resource,
            ManifestResourceConstraint::ExactNonFungibles(indexset!()),
        ))
        .withdraw_from_account(account, resource, dec!(0))
        .build();
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(manifest, [public_key.signature_proof()]);
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[derive(Copy, Clone)]
enum NextCallType {
    MethodCall,
    YieldToParent,
    YieldToChild,
}

fn run_return_two_resources_test(
    constraints: fn(ResourceAddress, ResourceAddress) -> ManifestResourceConstraints,
    exact: bool,
    next_call_type: NextCallType,
    expected_result: fn(ResourceAddress, ResourceAddress) -> Option<ResourceConstraintsError>,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let (transaction, resource1, resource2) = {
        match next_call_type {
            NextCallType::MethodCall => {
                let (pool, lp, resource1, resource2) =
                    create_pool(&mut ledger, account, &public_key);
                let mut builder = ManifestBuilder::new_v2()
                    .lock_fee_from_faucet()
                    .withdraw_from_account(account, lp, dec!(1))
                    .take_all_from_worktop(lp, "lp");

                builder = if exact {
                    builder.assert_next_call_returns_only(constraints(resource1, resource2))
                } else {
                    builder.assert_next_call_returns_include(constraints(resource1, resource2))
                };

                let manifest = builder
                    .with_bucket("lp", |builder, bucket| {
                        builder.call_method(
                            pool,
                            TWO_RESOURCE_POOL_REDEEM_IDENT,
                            TwoResourcePoolRedeemManifestInput { bucket },
                        )
                    })
                    .deposit_entire_worktop(account)
                    .build();
                (
                    TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
                        .finish_with_root_intent(manifest, [public_key.signature_proof()]),
                    resource1,
                    resource2,
                )
            }
            NextCallType::YieldToParent => {
                let resource1 = ledger.create_freely_mintable_fungible_resource(
                    OwnerRole::None,
                    None,
                    DIVISIBILITY_MAXIMUM,
                    account,
                );
                let resource2 = ledger.create_freely_mintable_fungible_resource(
                    OwnerRole::None,
                    None,
                    DIVISIBILITY_MAXIMUM,
                    account,
                );

                let mut builder = ManifestBuilder::new_subintent_v2();
                builder = if exact {
                    builder.assert_next_call_returns_only(constraints(resource1, resource2))
                } else {
                    builder.assert_next_call_returns_include(constraints(resource1, resource2))
                };
                let child = builder
                    .yield_to_parent(())
                    .deposit_entire_worktop(account)
                    .yield_to_parent(())
                    .build();

                let mut txn_builder =
                    TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
                let child = txn_builder.add_subintent(child, [public_key.signature_proof()]);
                let transaction = txn_builder.finish_with_root_intent(
                    ManifestBuilder::new_v2()
                        .lock_fee_from_faucet()
                        .use_child("child", child)
                        .yield_to_child("child", ())
                        .mint_fungible(resource1, dec!(10))
                        .mint_fungible(resource2, dec!(10))
                        .take_all_from_worktop(resource1, "resource1")
                        .take_all_from_worktop(resource2, "resource2")
                        .with_name_lookup(|builder, lookup| {
                            builder.yield_to_child(
                                "child",
                                (lookup.bucket("resource1"), lookup.bucket("resource2")),
                            )
                        })
                        .build(),
                    [public_key.signature_proof()],
                );
                (transaction, resource1, resource2)
            }

            NextCallType::YieldToChild => {
                let resource1 = ledger.create_freely_mintable_fungible_resource(
                    OwnerRole::None,
                    None,
                    DIVISIBILITY_MAXIMUM,
                    account,
                );
                let resource2 = ledger.create_freely_mintable_fungible_resource(
                    OwnerRole::None,
                    None,
                    DIVISIBILITY_MAXIMUM,
                    account,
                );

                let child = ManifestBuilder::new_subintent_v2()
                    .mint_fungible(resource1, dec!(10))
                    .mint_fungible(resource2, dec!(10))
                    .yield_to_parent(manifest_args!(ManifestExpression::EntireWorktop))
                    .build();

                let mut txn_builder =
                    TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
                let child = txn_builder.add_subintent(child, [public_key.signature_proof()]);
                let mut builder = ManifestBuilder::new_v2()
                    .lock_fee_from_faucet()
                    .use_child("child", child);
                builder = if exact {
                    builder.assert_next_call_returns_only(constraints(resource1, resource2))
                } else {
                    builder.assert_next_call_returns_include(constraints(resource1, resource2))
                };

                let manifest = builder
                    .yield_to_child("child", ())
                    .deposit_entire_worktop(account)
                    .build();
                let transaction =
                    txn_builder.finish_with_root_intent(manifest, [public_key.signature_proof()]);
                (transaction, resource1, resource2)
            }
        }
    };

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    if let Some(error) = expected_result(resource1, resource2) {
        receipt.expect_specific_failure(|e| {
            e.eq(&RuntimeError::SystemError(SystemError::IntentError(
                IntentError::AssertNextCallReturnsFailed(error.clone()),
            )))
        });
    } else {
        receipt.expect_commit_success();
    }
}

fn create_pool(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    account: ComponentAddress,
    pub_key: &Secp256k1PublicKey,
) -> (
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
    ResourceAddress,
) {
    let pool_resource1 = ledger.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        None,
        DIVISIBILITY_MAXIMUM,
        account,
    );
    let pool_resource2 = ledger.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        None,
        DIVISIBILITY_MAXIMUM,
        account,
    );

    let (pool_component, pool_unit_resource) = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                POOL_PACKAGE,
                TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                TwoResourcePoolInstantiateManifestInput {
                    resource_addresses: (pool_resource1.into(), pool_resource2.into()),
                    pool_manager_rule: AccessRule::AllowAll,
                    owner_role: OwnerRole::None,
                    address_reservation: None,
                },
            )
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        let commit_result = receipt.expect_commit_success();

        (
            commit_result.new_component_addresses()[0],
            commit_result.new_resource_addresses()[0],
        )
    };

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(pool_resource1, dec!("1000"))
        .mint_fungible(pool_resource2, dec!("1000"))
        .take_all_from_worktop(pool_resource1, "resource_1")
        .take_all_from_worktop(pool_resource2, "resource_2")
        .with_name_lookup(|builder, lookup| {
            let bucket1 = lookup.bucket("resource_1");
            let bucket2 = lookup.bucket("resource_2");
            builder.call_method(
                pool_component,
                TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                TwoResourcePoolContributeManifestInput {
                    buckets: (bucket1, bucket2),
                },
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    ledger
        .execute_manifest(manifest, vec![pub_key.signature_proof()])
        .expect_commit_success();

    (
        pool_component,
        pool_unit_resource,
        pool_resource1,
        pool_resource2,
    )
}
