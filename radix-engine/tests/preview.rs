use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::ExecutionConfig;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::SYS_FAUCET_COMPONENT;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::*;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::ValidationConfig;
use transaction::validation::{TestIntentHashManager, TransactionValidator};

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let (validated_transaction, preview_intent) = prepare_test_tx_and_preview_intent(&test_runner);

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_result = test_runner.execute_preview(preview_intent);
    let preview_receipt = preview_result.unwrap().receipt;
    preview_receipt.expect_commit_success();

    let receipt =
        test_runner.execute_transaction(&validated_transaction, &ExecutionConfig::standard());
    receipt.expect_commit_success();

    assert_eq!(
        preview_receipt.execution.fee_summary.cost_unit_consumed,
        receipt.execution.fee_summary.cost_unit_consumed
    );
}

fn prepare_test_tx_and_preview_intent(
    test_runner: &TestRunner<TypedInMemorySubstateStore>,
) -> (ValidatedTransaction, PreviewIntent) {
    let notary_priv_key = EcdsaPrivateKey::from_u64(2).unwrap();
    let tx_signer_priv_key = EcdsaPrivateKey::from_u64(3).unwrap();

    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network_id: NetworkDefinition::local_simulator().id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 99,
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary_priv_key.public_key(),
            notary_as_signatory: false,
            cost_unit_limit: 10_000_000,
            tip_percentage: 0,
        })
        .manifest(
            ManifestBuilder::new(&NetworkDefinition::local_simulator())
                .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
                .clear_auth_zone()
                .build(),
        )
        .sign(&tx_signer_priv_key)
        .notarize(&notary_priv_key)
        .build();

    let validated_transaction = TransactionValidator::validate(
        notarized_transaction.clone(),
        &TestIntentHashManager::new(),
        &ValidationConfig {
            network: NetworkDefinition::local_simulator(),
            current_epoch: 1,
            max_cost_unit_limit: 10_000_000,
            min_tip_percentage: 0,
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
