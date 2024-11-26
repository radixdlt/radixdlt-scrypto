use crate::prelude::*;

#[test]
fn should_not_be_able_to_use_subintent_when_verify_parent_access_rule_not_met() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(AccessRule::DenyAll)
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::IntentError(IntentError::VerifyParentFailed))
        )
    });
}

#[test]
fn should_be_able_to_use_subintent_when_verify_parent_access_rule_is_met() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(rule!(require(public_key.signature_proof())))
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_not_be_able_to_use_subintent_when_verify_parent_access_rule_not_met_two_layers() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let grandchild = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(rule!(require(public_key.signature_proof())))
            .yield_to_parent(())
            .build(),
        [],
    );

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .use_child("grandchild", grandchild)
            .yield_to_child("grandchild", ())
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::IntentError(IntentError::VerifyParentFailed))
        )
    });
}

#[test]
fn should_be_able_to_use_subintent_when_verify_parent_access_rule_is_met_two_layers() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, _account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let grandchild = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(rule!(require(public_key.signature_proof())))
            .yield_to_parent(())
            .build(),
        [],
    );

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .use_child("grandchild", grandchild)
            .yield_to_child("grandchild", ())
            .yield_to_parent(())
            .build(),
        [public_key.signature_proof()],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_fee_from_faucet()
            .yield_to_child("child", ())
            .build(),
        [],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_use_subintent_when_verify_parent_access_rule_is_met_on_second_yield() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .yield_to_parent(())
            .verify_parent(rule!(require_amount(dec!(10), XRD)))
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .create_proof_from_account_of_amount(account, XRD, dec!(10))
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn verify_parent_should_only_work_against_proofs_in_parent_intent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // We will create a complex transaction with lots of intents.
    // Naming convention: subintent XXX will have children XXXa, XXXb, XXXc, etc.
    // Each intent will be signed by its own key.

    let keys = indexmap! {
        "aaa" => ledger.new_allocated_account().0,
        "aaba" => ledger.new_allocated_account().0,
        "aab" => ledger.new_allocated_account().0,
        "aac" => ledger.new_allocated_account().0,
        "aa" => ledger.new_allocated_account().0,
        "ab" => ledger.new_allocated_account().0,
        "a" => ledger.new_allocated_account().0,
        "ba" => ledger.new_allocated_account().0,
        "b" => ledger.new_allocated_account().0,
        "c" => ledger.new_allocated_account().0,
        "ROOT" => ledger.new_allocated_account().0,
    };

    // We will set all their manifests to be simple/trivial, except the AAB manifest
    // which will be set to call "verify parent" with each of the keys.
    // The transaction should only pass if the AA key is used (its parent).
    for (key_name_to_assert, key_to_assert) in keys.iter() {
        let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

        let aaa = builder.add_simple_subintent([], [keys["aaa"].signature_proof()]);
        let aaba = builder.add_simple_subintent([], [keys["aaba"].signature_proof()]);
        let aab = builder.add_tweaked_simple_subintent(
            [aaba],
            [keys["aab"].signature_proof()],
            |builder| builder.verify_parent(rule!(require(key_to_assert.signature_proof()))),
        );
        let aac = builder.add_simple_subintent([], [keys["aac"].signature_proof()]);
        let aa = builder.add_simple_subintent([aaa, aab, aac], [keys["aa"].signature_proof()]);
        let ab = builder.add_simple_subintent([], [keys["ab"].signature_proof()]);
        let a = builder.add_simple_subintent([aa, ab], [keys["a"].signature_proof()]);
        let ba = builder.add_simple_subintent([], [keys["ba"].signature_proof()]);
        let b = builder.add_simple_subintent([ba], [keys["b"].signature_proof()]);
        let c = builder.add_simple_subintent([], [keys["c"].signature_proof()]);
        let transaction =
            builder.finish_with_simple_root_intent([a, b, c], [keys["ROOT"].signature_proof()]);

        let receipt = ledger.execute_test_transaction(transaction);

        // ASSERT
        if *key_name_to_assert == "aa" {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|e| {
                matches!(
                    e,
                    RuntimeError::SystemError(SystemError::IntentError(
                        IntentError::VerifyParentFailed
                    ))
                )
            });
        }
    }
}
