use radix_common::prelude::{FromPublicKey, NonFungibleGlobalId};
use radix_engine::transaction::ExecutionConfig;
use radix_rust::btreeset;
use radix_transactions::builder::ManifestV2Builder;
use radix_transactions::model::{ManifestIntent, TestTransaction};
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn simple_subintent_transaction_should_work() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestV2Builder::new_v2()
                .lock_standard_test_fee(account)
                .yield_to_child(ManifestIntent(0), ())
                .build();
            (manifest, ledger.next_transaction_nonce(), vec![1])
        },
        {
            let manifest = ManifestV2Builder::new_v2().build();
            (manifest, ledger.next_transaction_nonce(), vec![])
        },
    ];

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
}
