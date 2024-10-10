use scrypto_test::prelude::*;

#[test]
fn can_send_resources_to_child_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .assert_worktop_contains(XRD, dec!(10))
            .deposit_entire_worktop(account)
            .yield_to_parent(())
            .build(),
        [public_key.signature_proof()],
    );
    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .withdraw_from_account(account, XRD, dec!(10))
            .take_all_from_worktop(XRD, "xrd")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_child("child", (lookup.bucket("xrd"),))
            })
            .build(),
        [public_key.signature_proof()],
    );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_send_resources_to_parent_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .withdraw_from_account(account, XRD, dec!(10))
            .take_all_from_worktop(XRD, "xrd")
            .with_name_lookup(|builder, lookup| builder.yield_to_parent((lookup.bucket("xrd"),)))
            .build(),
        [public_key.signature_proof()],
    );
    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .assert_worktop_contains(XRD, dec!(10))
            .deposit_entire_worktop(account)
            .build(),
        [public_key.signature_proof()],
    );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_send_and_receive_resources_as_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .take_all_from_worktop(XRD, "xrd")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_parent(manifest_args!(lookup.bucket("xrd")))
            })
            .build(),
        [public_key.signature_proof()],
    );
    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .withdraw_from_account(account, XRD, dec!(10))
            .take_all_from_worktop(XRD, "xrd")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_child("child", manifest_args!(lookup.bucket("xrd")))
            })
            .assert_worktop_contains(XRD, dec!(10))
            .deposit_entire_worktop(account)
            .build(),
        [public_key.signature_proof()],
    );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}
