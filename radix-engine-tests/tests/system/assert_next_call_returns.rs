use radix_engine::blueprints::pool::v1::constants::TWO_RESOURCE_POOL_BLUEPRINT_IDENT;
use scrypto_test::prelude::*;

#[test]
fn when_more_is_returned_assert_next_call_returns_include_should_succeed() {
    run_return_two_resources_test(
        |resource1, _resource2| {
            ManifestResourceConstraints::new().with(
                resource1,
                ManifestResourceConstraint::AtLeastAmount(dec!(1)),
            )
        },
        false,
        None,
    );
}

#[test]
fn when_more_is_returned_assert_next_call_returns_only_should_fail() {
    run_return_two_resources_test(
        |resource1, _resource2| {
            ManifestResourceConstraints::new().with(
                resource1,
                ManifestResourceConstraint::AtLeastAmount(dec!(1)),
            )
        },
        true,
        Some(ManifestResourceConstraintsError::UnwantedResourcesExist),
    );
}

#[test]
fn when_exact_is_returned_assert_next_call_returns_include_should_succeed() {
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
        None,
    );
}

#[test]
fn when_exact_is_returned_assert_next_call_returns_only_should_succeed() {
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
        None,
    );
}

#[test]
fn when_less_is_returned_assert_next_call_returns_include_should_fail() {
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
        Some(ManifestResourceConstraintsError::ResourceConstraint(
            ResourceConstraintError::ExpectedAtLeastAmount {
                expected_at_least_amount: dec!(1),
                actual_amount: dec!(0),
            },
        )),
    );
}

#[test]
fn when_less_is_returned_assert_next_call_returns_only_should_fail() {
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
        Some(ManifestResourceConstraintsError::ResourceConstraint(
            ResourceConstraintError::ExpectedAtLeastAmount {
                expected_at_least_amount: dec!(1),
                actual_amount: dec!(0),
            },
        )),
    );
}

#[test]
fn when_empty_constraints_on_assert_next_call_returns_include_should_succeed() {
    run_return_two_resources_test(
        |_resource1, _resource2| ManifestResourceConstraints::new(),
        false,
        None,
    );
}

#[test]
fn when_empty_constraints_on_assert_next_call_returns_only_should_fail() {
    run_return_two_resources_test(
        |_resource1, _resource2| ManifestResourceConstraints::new(),
        true,
        Some(ManifestResourceConstraintsError::UnwantedResourcesExist),
    );
}

#[test]
fn when_extra_zero_constraints_on_assert_next_call_returns_include_should_succeed() {
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
        None,
    );
}

#[test]
fn when_extra_zero_constraints_on_assert_next_call_returns_only_should_succeed() {
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
        None,
    );
}

fn run_return_two_resources_test(
    constraints: fn(ResourceAddress, ResourceAddress) -> ManifestResourceConstraints,
    exact: bool,
    result: Option<ManifestResourceConstraintsError>,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (pool, lp, resource1, resource2) = create_pool(&mut ledger, account, &public_key);

    // Act
    let transaction = {
        let mut builder = ManifestBuilder::new_v2()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, lp, dec!(1))
            .take_all_from_worktop(lp, "lp");

        builder = if exact {
            builder.assert_next_call_returns_only(constraints(resource1, resource2))
        } else {
            builder.assert_next_call_returns_include(constraints(resource1, resource2))
        };

        builder
            .with_bucket("lp", |builder, bucket| {
                builder.call_method(
                    pool,
                    TWO_RESOURCE_POOL_REDEEM_IDENT,
                    TwoResourcePoolRedeemManifestInput { bucket },
                )
            })
            .deposit_entire_worktop(account)
            .build()
    };
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(transaction, [public_key.signature_proof()]);
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    if let Some(error) = result {
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
                    resource_addresses: (pool_resource1, pool_resource2),
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
