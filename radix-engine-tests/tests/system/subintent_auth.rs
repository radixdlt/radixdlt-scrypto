use radix_common::prelude::{FromPublicKey, NonFungibleGlobalId, XRD};
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::transaction::ExecutionConfig;
use radix_engine_interface::macros::dec;
use radix_rust::btreeset;
use radix_transactions::builder::ManifestV2Builder;
use radix_transactions::model::{ManifestIntent, TestTransaction};
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn should_not_be_able_to_use_root_auth_in_subintent() {
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

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestV2Builder::new_v2()
                .withdraw_from_account(account, XRD, dec!(10))
                .deposit_entire_worktop(account)
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset!(),
            )
        },
    ];

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn should_be_able_to_use_separate_auth_in_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestV2Builder::new_v2()
                .lock_standard_test_fee(account)
                .yield_to_child(ManifestIntent(0), ())
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestV2Builder::new_v2()
                .withdraw_from_account(account2, XRD, dec!(10))
                .deposit_entire_worktop(account2)
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key2)],
            )
        },
    ];

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
}
