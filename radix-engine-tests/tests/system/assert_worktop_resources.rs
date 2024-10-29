use scrypto_test::prelude::*;

#[test]
fn when_more_is_returned_assert_worktop_resources_include_should_succeed() {
    run_worktop_two_resources_test(
        |resource1, _resource2| {
            ManifestResourceConstraints::new().with(
                resource1,
                ManifestResourceConstraint::AtLeastAmount(dec!(1)),
            )
        },
        false,
        |_, _| None,
    );
}

#[test]
fn when_more_is_returned_assert_worktop_resources_only_should_fail() {
    run_worktop_two_resources_test(
        |resource1, _resource2| {
            ManifestResourceConstraints::new().with(
                resource1,
                ManifestResourceConstraint::AtLeastAmount(dec!(1)),
            )
        },
        true,
        |_, resource2| {
            Some(
                ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                    resource_address: resource2,
                },
            )
        },
    );
}

#[test]
fn when_exact_is_returned_assert_next_call_returns_include_should_succeed() {
    run_worktop_two_resources_test(
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
        |_, _| None,
    );
}

#[test]
fn when_exact_is_returned_assert_next_call_returns_only_should_succeed() {
    run_worktop_two_resources_test(
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
        |_, _| None,
    );
}

#[test]
fn when_less_is_returned_assert_next_call_returns_include_should_fail() {
    run_worktop_two_resources_test(
        |resource1, _resource2| {
            ManifestResourceConstraints::new()
                .with(
                    resource1,
                    ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                )
                .with(XRD, ManifestResourceConstraint::AtLeastAmount(dec!(1)))
        },
        false,
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

#[test]
fn when_less_is_returned_assert_next_call_returns_only_should_fail() {
    run_worktop_two_resources_test(
        |resource1, _resource2| {
            ManifestResourceConstraints::new()
                .with(
                    resource1,
                    ManifestResourceConstraint::AtLeastAmount(dec!(1)),
                )
                .with(XRD, ManifestResourceConstraint::AtLeastAmount(dec!(1)))
        },
        true,
        |_, resource2| {
            Some(
                ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                    resource_address: resource2,
                },
            )
        },
    );
}

#[test]
fn when_empty_constraints_on_assert_next_call_returns_include_should_succeed() {
    run_worktop_two_resources_test(
        |_resource1, _resource2| ManifestResourceConstraints::new(),
        false,
        |_, _| None,
    );
}

#[test]
fn when_empty_constraints_on_assert_next_call_returns_only_should_fail() {
    run_worktop_two_resources_test(
        |_resource1, _resource2| ManifestResourceConstraints::new(),
        true,
        |resource1, _resource2| {
            Some(
                ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                    resource_address: resource1,
                },
            )
        },
    );
}

#[test]
fn when_extra_zero_constraints_on_assert_next_call_returns_include_should_succeed() {
    run_worktop_two_resources_test(
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
        |_, _| None,
    );
}

#[test]
fn when_extra_zero_constraints_on_assert_next_call_returns_only_should_succeed() {
    run_worktop_two_resources_test(
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
        |_, _| None,
    );
}

#[test]
fn when_withdrawing_zero_amount_with_zero_constraints_on_assert_worktop_resources_returns_only_should_succeed(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_from_faucet()
        .assert_worktop_resources_only(
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
fn when_withdrawing_zero_non_fungibles_with_zero_constraints_on_assert_worktop_resources_only_should_succeed(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_from_faucet()
        .assert_worktop_resources_only(ManifestResourceConstraints::new().with(
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

fn run_worktop_two_resources_test(
    constraints: fn(ResourceAddress, ResourceAddress) -> ManifestResourceConstraints,
    exact: bool,
    expected_result: fn(ResourceAddress, ResourceAddress) -> Option<ResourceConstraintsError>,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let (transaction, resource1, resource2) = {
        let (resource1, resource2) = create_two_resources(&mut ledger, account);
        let builder = ManifestBuilder::new_v2()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource1, dec!(1))
            .withdraw_from_account(account, resource2, dec!(1));

        let builder = if exact {
            builder.assert_worktop_resources_only(constraints(resource1, resource2))
        } else {
            builder.assert_worktop_resources_include(constraints(resource1, resource2))
        };

        let manifest = builder.deposit_entire_worktop(account).build();
        (
            TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
                .finish_with_root_intent(manifest, [public_key.signature_proof()]),
            resource1,
            resource2,
        )
    };

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    if let Some(error) = expected_result(resource1, resource2) {
        receipt.expect_specific_failure(|e| {
            e.eq(&RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed(error.clone())),
            ))
        });
    } else {
        receipt.expect_commit_success();
    }
}

fn create_two_resources(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    account: ComponentAddress,
) -> (ResourceAddress, ResourceAddress) {
    let resource1 = ledger.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(10)),
        DIVISIBILITY_MAXIMUM,
        account,
    );
    let resource2 = ledger.create_freely_mintable_and_burnable_fungible_resource(
        OwnerRole::None,
        Some(dec!(10)),
        DIVISIBILITY_MAXIMUM,
        account,
    );

    (resource1, resource2)
}
