#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::transaction::ExecutionParameters;
use scrypto::core::Network;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::*;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::ValidationParameters;
use transaction::validation::{TestIntentHashManager, TransactionValidator};

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let (validated_transaction, preview_intent) = prepare_test_tx_and_preview_intent(&test_runner);

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_result = test_runner.execute_preview(preview_intent);
    let preview_receipt = preview_result.unwrap().receipt;
    preview_receipt.expect_success();

    let receipt =
        test_runner.execute_transaction(&validated_transaction, &ExecutionParameters::default());
    receipt.expect_success();

    assert_eq!(
        preview_receipt.transaction_fee.cost_unit_consumed,
        receipt.transaction_fee.cost_unit_consumed
    );
}

fn prepare_test_tx_and_preview_intent(
    test_runner: &TestRunner<InMemorySubstateStore>,
) -> (ValidatedTransaction, PreviewIntent) {
    let notary_priv_key = EcdsaPrivateKey::from_u64(2).unwrap();
    let tx_signer_priv_key = EcdsaPrivateKey::from_u64(3).unwrap();

    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network: Network::LocalSimulator,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 99,
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary_priv_key.public_key(),
            notary_as_signatory: false,
            cost_unit_limit: 10_000_000,
            tip_bps: 0,
        })
        .manifest(
            ManifestBuilder::new(Network::LocalSimulator)
                .clear_auth_zone()
                .build(),
        )
        .sign(&tx_signer_priv_key)
        .notarize(&notary_priv_key)
        .build();

    let validated_transaction = TransactionValidator::validate(
        notarized_transaction.clone(),
        &TestIntentHashManager::new(),
        &ValidationParameters {
            network: Network::LocalSimulator,
            current_epoch: 1,
            max_cost_unit_limit: 10_000_000,
            min_tip_bps: 0,
        },
    )
    .unwrap();

    let preview_intent = PreviewIntent {
        intent: notarized_transaction.signed_intent.intent.clone(),
        signer_public_keys: notarized_transaction
            .signed_intent
            .intent_signatures
            .iter()
            .map(|p| p.0)
            .collect(),
        flags: PreviewFlags {
            unlimited_loan: true,
        },
    };

    (validated_transaction, preview_intent)
}
