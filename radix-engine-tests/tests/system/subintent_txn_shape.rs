use scrypto_test::prelude::*;

#[test]
fn simple_subintent_should_work() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
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
fn multiple_flat_subintents_should_work() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let children = (0..4)
        .into_iter()
        .map(|_| {
            builder.add_subintent(
                ManifestBuilder::new_subintent_v2()
                    .yield_to_parent(())
                    .build(),
                [],
            )
        })
        .collect::<Vec<_>>();

    let mut root_manifest_builder = ManifestBuilder::new_v2().lock_standard_test_fee(account);

    for (index, child_hash) in children.into_iter().enumerate() {
        let child_name = format!("child{index}");
        root_manifest_builder = root_manifest_builder
            .use_child(&child_name, child_hash)
            .yield_to_child(&child_name, ());
    }

    let transaction = builder.finish_with_root_intent(
        root_manifest_builder.build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn multiple_deep_subintents_should_work() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    // Create deepest child
    let mut child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .yield_to_parent(())
            .build(),
        [],
    );

    // Create middle-layer children
    for _ in 0..3 {
        child = builder.add_subintent(
            ManifestBuilder::new_subintent_v2()
                .use_child("child", child)
                .yield_to_child("child", ())
                .yield_to_parent(())
                .build(),
            [],
        );
    }

    // Create top-level root manifest
    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .lock_standard_test_fee(account)
            .use_child("child", child)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}
