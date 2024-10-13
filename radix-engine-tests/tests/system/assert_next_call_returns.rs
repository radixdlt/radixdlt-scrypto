use radix_engine::blueprints::pool::v1::constants::TWO_RESOURCE_POOL_BLUEPRINT_IDENT;
use scrypto_test::prelude::*;

#[test]
fn assert_correct_next_call_returns_include_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_fungible_resource(dec!(100000000), DIVISIBILITY_MAXIMUM, account);
    let amount = dec!(10);

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .assert_next_call_returns_include(ManifestResourceConstraints::new()
                    .with(resource, ManifestResourceConstraint::ExactAmount(amount)))
                .withdraw_from_account(account, resource, amount)
                .deposit_entire_worktop(account)
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn assert_empty_amount_next_call_returns_include_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .assert_next_call_returns_include(ManifestResourceConstraints::new()
                    .with(XRD, ManifestResourceConstraint::ExactAmount(dec!(0))))
                .deposit_entire_worktop(account)
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn assert_empty_constraints_next_call_returns_include_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_fungible_resource(dec!(100000000), DIVISIBILITY_MAXIMUM, account);
    let amount = dec!(10);

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .assert_next_call_returns_include(ManifestResourceConstraints::new())
                .withdraw_from_account(account, resource, amount)
                .deposit_entire_worktop(account)
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn assert_correct_multiple_constraints_next_call_returns_include_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (pool, lp, resource1, resource2) = create_pool(&mut ledger, account, &public_key);

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .withdraw_from_account(account, lp, dec!(1))
                .take_all_from_worktop(lp, "lp")
                .assert_next_call_returns_include(ManifestResourceConstraints::new()
                    .with(resource1, ManifestResourceConstraint::AtLeastAmount(dec!(1)))
                    .with(resource2, ManifestResourceConstraint::AtLeastAmount(dec!(1)))
                )
                .with_bucket("lp", |builder, bucket| {
                    builder.call_method(pool, TWO_RESOURCE_POOL_REDEEM_IDENT, TwoResourcePoolRedeemManifestInput {
                        bucket
                    })
                })
                .deposit_entire_worktop(account)
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}


#[test]
fn assert_incorrect_next_call_returns_include_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_fungible_resource(dec!(100000000), DIVISIBILITY_MAXIMUM, account);
    let amount = dec!(10);

    // Act
    let transaction = TestTransaction::new_v2_builder(ledger.next_transaction_nonce())
        .finish_with_root_intent(
            ManifestBuilder::new_v2()
                .lock_fee_from_faucet()
                .assert_next_call_returns_include(ManifestResourceConstraints::new()
                    .with(XRD, ManifestResourceConstraint::ExactAmount(amount)))
                .withdraw_from_account(account, resource, amount)
                .deposit_entire_worktop(account)
                .build(),
            [public_key.signature_proof()],
        );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(SystemError::IntentError(IntentError::AssertNextCallReturnsFailed(
        ManifestResourceConstraintsError::ResourceConstraint(ResourceConstraintError::ExpectedExactAmount { .. }))))));
}

fn create_pool(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    account: ComponentAddress,
    pub_key: &Secp256k1PublicKey,
) -> (ComponentAddress, ResourceAddress, ResourceAddress, ResourceAddress) {
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

    (pool_component, pool_unit_resource, pool_resource1, pool_resource2)
}