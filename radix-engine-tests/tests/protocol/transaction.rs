use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::transaction::*;
use radix_engine::updates::ProtocolVersion;
use scrypto_test::prelude::*;

#[test]
fn bottlenose_protocol_should_not_support_v2_transactions() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Bottlenose))
        .build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = (0..2)
        .into_iter()
        .map(|_| {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .build();
            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        })
        .collect();
    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_specific_rejection(|e| matches!(e, RejectionReason::TransactionNotYetSupported));
}
