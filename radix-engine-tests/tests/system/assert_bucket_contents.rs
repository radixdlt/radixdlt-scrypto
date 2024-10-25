use scrypto_test::prelude::*;

#[test]
fn asserting_correct_non_zero_amount_should_succceed() {
    test_fungible_constraint(dec!(10), ManifestResourceConstraint::NonZeroAmount, None)
}

#[test]
fn asserting_incorrect_non_zero_amount_should_fail() {
    test_fungible_constraint(
        dec!(0),
        ManifestResourceConstraint::NonZeroAmount,
        Some(ResourceConstraintError::ExpectedNonZeroAmount),
    )
}

#[test]
fn asserting_correct_exact_amount_should_succceed() {
    test_fungible_constraint(
        dec!(10),
        ManifestResourceConstraint::ExactAmount(dec!(10)),
        None,
    )
}

#[test]
fn asserting_incorrect_exact_amount_should_fail() {
    let actual_amount = dec!(10);
    let expected_exact_amount = dec!(10) - Decimal::from_attos(I192::ONE);

    test_fungible_constraint(
        actual_amount,
        ManifestResourceConstraint::ExactAmount(expected_exact_amount),
        Some(ResourceConstraintError::ExpectedExactAmount {
            actual_amount,
            expected_amount: expected_exact_amount,
        }),
    )
}

#[test]
fn asserting_correct_at_least_amount_should_succceed() {
    test_fungible_constraint(
        dec!(10),
        ManifestResourceConstraint::AtLeastAmount(dec!(10)),
        None,
    )
}

#[test]
fn asserting_incorrect_at_least_amount_should_fail() {
    let actual_amount = dec!(10);
    let expected_atleast_amount = dec!(10) + Decimal::from_attos(I192::ONE);

    test_fungible_constraint(
        actual_amount,
        ManifestResourceConstraint::AtLeastAmount(expected_atleast_amount),
        Some(ResourceConstraintError::ExpectedAtLeastAmount {
            actual_amount,
            expected_at_least_amount: expected_atleast_amount,
        }),
    )
}

#[test]
fn asserting_correct_at_least_non_fungibles_should_succceed() {
    test_non_fungible_constraint(
        vec![1, 2],
        ManifestResourceConstraint::AtLeastNonFungibles(indexset!(NonFungibleLocalId::from(1))),
        None,
    )
}

#[test]
fn asserting_incorrect_at_least_non_fungibles_should_fail() {
    let actual_ids = vec![1];
    let expected_at_least_ids = indexset!(NonFungibleLocalId::from(2));

    test_non_fungible_constraint(
        actual_ids.clone(),
        ManifestResourceConstraint::AtLeastNonFungibles(expected_at_least_ids.clone()),
        Some(ResourceConstraintError::NonFungibleMissing {
            missing_id: NonFungibleLocalId::from(2),
        }),
    )
}

#[test]
fn asserting_correct_exact_non_fungibles_should_succceed() {
    test_non_fungible_constraint(
        vec![1, 2],
        ManifestResourceConstraint::ExactNonFungibles(indexset!(
            NonFungibleLocalId::from(1),
            NonFungibleLocalId::from(2)
        )),
        None,
    )
}

#[test]
fn asserting_incorrect_exact_non_fungibles_should_fail() {
    let actual_ids = vec![1, 2];
    let expected_exact_ids = indexset!(
        NonFungibleLocalId::from(1),
        NonFungibleLocalId::from(2),
        NonFungibleLocalId::from(3)
    );

    test_non_fungible_constraint(
        actual_ids.clone(),
        ManifestResourceConstraint::ExactNonFungibles(expected_exact_ids.clone()),
        Some(ResourceConstraintError::NonFungibleMissing {
            missing_id: NonFungibleLocalId::from(3),
        }),
    )
}

#[test]
fn asserting_correct_fungible_general_constraints_should_succeed() {
    let amount = dec!(10);

    let lower_bounds = [LowerBound::NonZero, LowerBound::Inclusive(amount)];

    let upper_bounds = [UpperBound::Unbounded, UpperBound::Inclusive(amount)];

    for lower_bound in lower_bounds {
        for upper_bound in upper_bounds {
            let constraint = GeneralResourceConstraint {
                required_ids: Default::default(),
                allowed_ids: AllowedIds::Any,
                lower_bound,
                upper_bound,
            };
            test_fungible_constraint(
                amount,
                ManifestResourceConstraint::General(constraint),
                None,
            );
        }
    }
}

#[test]
fn asserting_correct_non_fungible_general_constraints_should_succeed() {
    let actual_ids = vec![1, 2];

    let lower_bounds = [LowerBound::NonZero, LowerBound::Inclusive(Decimal::from(2))];

    let upper_bounds = [
        UpperBound::Unbounded,
        UpperBound::Inclusive(Decimal::from(2)),
    ];

    let required_ids_list = [
        indexset!(NonFungibleLocalId::from(1)),
        indexset!(NonFungibleLocalId::from(1), NonFungibleLocalId::from(2)),
    ];

    let allowed_ids_list = [
        AllowedIds::Any,
        AllowedIds::Allowlist(indexset!(
            NonFungibleLocalId::from(1),
            NonFungibleLocalId::from(2)
        )),
    ];

    for lower_bound in lower_bounds {
        for upper_bound in upper_bounds {
            for required_ids in &required_ids_list {
                for allowed_ids in &allowed_ids_list {
                    let constraint = GeneralResourceConstraint {
                        required_ids: required_ids.clone(),
                        allowed_ids: allowed_ids.clone(),
                        lower_bound,
                        upper_bound,
                    };
                    test_non_fungible_constraint(
                        actual_ids.clone(),
                        ManifestResourceConstraint::General(constraint),
                        None,
                    );
                }
            }
        }
    }
}

#[test]
fn asserting_incorrect_fungible_lower_bound_general_constraint_should_fail() {
    let amount = dec!(10);
    let lower_bound = dec!(10) + Decimal::from_attos(I192::ONE);
    let constraint = GeneralResourceConstraint {
        required_ids: Default::default(),
        lower_bound: LowerBound::Inclusive(lower_bound),
        upper_bound: UpperBound::Unbounded,
        allowed_ids: AllowedIds::Any,
    };
    test_fungible_constraint(
        amount,
        ManifestResourceConstraint::General(constraint),
        Some(ResourceConstraintError::ExpectedAtLeastAmount {
            expected_at_least_amount: lower_bound,
            actual_amount: amount,
        }),
    );
}

#[test]
fn asserting_incorrect_fungible_upper_bound_general_constraint_should_fail() {
    let amount = dec!(10);
    let upper_bound = dec!(10) - Decimal::from_attos(I192::ONE);
    let constraint = GeneralResourceConstraint {
        required_ids: Default::default(),
        lower_bound: LowerBound::Inclusive(Decimal::zero()),
        upper_bound: UpperBound::Inclusive(upper_bound),
        allowed_ids: AllowedIds::Any,
    };
    test_fungible_constraint(
        amount,
        ManifestResourceConstraint::General(constraint),
        Some(ResourceConstraintError::ExpectedAtMostAmount {
            expected_at_most_amount: upper_bound,
            actual_amount: amount,
        }),
    );
}

#[test]
fn asserting_incorrect_non_fungible_required_ids_general_constraint_should_fail() {
    let actual_ids = vec![1, 2];
    let constraint = GeneralResourceConstraint {
        required_ids: indexset!(NonFungibleLocalId::from(3)),
        lower_bound: LowerBound::Inclusive(Decimal::from(1)),
        upper_bound: UpperBound::Unbounded,
        allowed_ids: AllowedIds::Any,
    };
    test_non_fungible_constraint(
        actual_ids.clone(),
        ManifestResourceConstraint::General(constraint),
        Some(ResourceConstraintError::NonFungibleMissing {
            missing_id: NonFungibleLocalId::from(3),
        }),
    );
}

#[test]
fn asserting_incorrect_non_fungible_allowed_ids_general_constraint_should_fail() {
    let actual_ids = vec![1, 3];
    let constraint = GeneralResourceConstraint {
        required_ids: indexset!(),
        lower_bound: LowerBound::NonZero,
        upper_bound: UpperBound::Inclusive(Decimal::from(2)),
        allowed_ids: AllowedIds::Allowlist(indexset!(
            NonFungibleLocalId::from(3),
            NonFungibleLocalId::from(4)
        )),
    };
    test_non_fungible_constraint(
        actual_ids.clone(),
        ManifestResourceConstraint::General(constraint),
        Some(ResourceConstraintError::NonFungibleNotAllowed {
            disallowed_id: NonFungibleLocalId::from(1),
        }),
    );
}

#[test]
fn asserting_correct_empty_bucket_general_constraints_should_succeed() {
    let amount = dec!(0);

    let upper_bounds = [UpperBound::Unbounded, UpperBound::Inclusive(amount)];
    let allowed_ids_list = [AllowedIds::Any, AllowedIds::Allowlist(Default::default())];

    for upper_bound in upper_bounds {
        for allowed_ids in &allowed_ids_list {
            let constraint = GeneralResourceConstraint {
                required_ids: Default::default(),
                lower_bound: LowerBound::Inclusive(amount),
                upper_bound,
                allowed_ids: allowed_ids.clone(),
            };
            test_fungible_constraint(
                amount,
                ManifestResourceConstraint::General(constraint),
                None,
            );
        }
    }
}

#[test]
fn asserting_incorrect_empty_bucket_lower_bound_general_constraint_should_fail() {
    let amount = dec!(0);
    let constraint = GeneralResourceConstraint {
        required_ids: Default::default(),
        lower_bound: LowerBound::NonZero,
        upper_bound: UpperBound::Unbounded,
        allowed_ids: AllowedIds::Any,
    };
    test_fungible_constraint(
        amount,
        ManifestResourceConstraint::General(constraint),
        Some(ResourceConstraintError::ExpectedNonZeroAmount),
    );
}

fn test_fungible_constraint(
    bucket_amount: Decimal,
    constraint: ManifestResourceConstraint,
    expected_error: Option<ResourceConstraintError>,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_fungible_resource(dec!(100000000), DIVISIBILITY_MAXIMUM, account);

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .withdraw_from_account(account, resource, bucket_amount)
                .take_all_from_worktop(resource, "bucket")
                .assert_bucket_contents("bucket", constraint)
                .deposit(account, "bucket")
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    if let Some(constraint_error) = expected_error {
        let expected_error = RuntimeError::SystemError(SystemError::IntentError(
            IntentError::AssertBucketContentsFailed(constraint_error),
        ));
        receipt.expect_specific_failure(|e| e.eq(&expected_error))
    } else {
        receipt.expect_commit_success();
    }
}

fn test_non_fungible_constraint(
    non_fungibles: Vec<u64>,
    constraint: ManifestResourceConstraint,
    expected_error: Option<ResourceConstraintError>,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_non_fungible_resource(account);

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .withdraw_non_fungibles_from_account(
                    account,
                    resource,
                    non_fungibles
                        .into_iter()
                        .map(|i| NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(i))),
                )
                .take_all_from_worktop(resource, "bucket")
                .assert_bucket_contents("bucket", constraint)
                .deposit(account, "bucket")
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    if let Some(constraint_error) = expected_error {
        let expected_error = RuntimeError::SystemError(SystemError::IntentError(
            IntentError::AssertBucketContentsFailed(constraint_error),
        ));
        receipt.expect_specific_failure(|e| e.eq(&expected_error))
    } else {
        receipt.expect_commit_success();
    }
}
