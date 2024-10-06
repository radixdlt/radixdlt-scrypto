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
        Some(ResourceConstraintError::NonZeroAmount),
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
        Some(ResourceConstraintError::ExactAmount {
            actual_amount,
            expected_exact_amount,
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
        Some(ResourceConstraintError::AtLeastAmount {
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
        Some(ResourceConstraintError::AtLeastNonFungibles {
            actual_ids: Box::new(actual_ids
                .into_iter()
                .map(NonFungibleLocalId::from)
                .collect()),
            expected_at_least_ids: Box::new(expected_at_least_ids),
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
        Some(ResourceConstraintError::ExactNonFungibles {
            actual_ids: Box::new(actual_ids
                .into_iter()
                .map(NonFungibleLocalId::from)
                .collect()),
            expected_exact_ids: Box::new(expected_exact_ids),
        }),
    )
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
