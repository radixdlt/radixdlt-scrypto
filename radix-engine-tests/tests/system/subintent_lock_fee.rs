use radix_common::prelude::{FromPublicKey, NonFungibleGlobalId};
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::transaction::ExecutionConfig;
use radix_rust::btreeset;
use radix_transactions::builder::ResolvableArguments;
use radix_transactions::manifest::YieldToChild;
use radix_transactions::model::{ManifestNamedIntentIndex, TestTransaction};
use radix_transactions::prelude::ManifestBuilder;
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn should_not_be_able_to_lock_fee_in_a_child_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
    ];

    let receipt = ledger.execute_test_transaction(TestTransaction::new_v2_from_nonce(intents));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::CannotLockFeeInChildSubintent(..))
        )
    });
}
