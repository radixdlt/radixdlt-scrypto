use scrypto_test::prelude::*;

#[test]
fn bottlenose_protocol_should_not_support_v2_transactions() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Bottlenose))
        .build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
    let child_one = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .yield_to_parent(())
            .build(),
        [],
    );
    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child_one)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );
    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_rejection(|e| matches!(e, RejectionReason::SubintentsNotYetSupported));
}
