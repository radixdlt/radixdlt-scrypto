#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::{RuntimeError, TransactionCostCounterConfig, TransactionExecutorConfig};
use radix_engine::fee::{CostUnitCounterError, DEFAULT_MAX_TRANSACTION_COST};
use scrypto::core::Network;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::*;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::{TestEpochManager, TestIntentHashManager, TransactionValidator};

#[test]
fn test_transaction_preview_cost_estimate() {
    let mut test_runner = TestRunner::new(true);

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
        &TestEpochManager::new(0),
    )
    .unwrap();

    // Just to check that the test transaction really fails with OutOfCostUnit if loan isn't repaid
    let receipt = test_runner.execute_transaction(
        &validated_transaction,
        TransactionExecutorConfig::new(
            true,
            TransactionCostCounterConfig::SystemLoanAndMaxCost {
                system_loan_amount: 10_000, // Too little
                max_transaction_cost: DEFAULT_MAX_TRANSACTION_COST,
            },
        ),
    );

    receipt.expect_err(|e| {
        matches!(
            e,
            RuntimeError::CostingError(CostUnitCounterError::OutOfCostUnit)
        )
    });

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

    // Transaction preview should succeed
    let preview_result = test_runner.execute_preview(preview_intent);
    assert!(preview_result.is_ok());
    let preview_receipt = preview_result.unwrap().receipt;
    preview_receipt.expect_success();

    // Real transaction should also succeed
    let receipt = test_runner.execute_transaction(
        &validated_transaction,
        TransactionExecutorConfig::default(false),
    );
    receipt.expect_success();

    // And the preview result should provide an accurate cost estimate
    assert_eq!(
        preview_receipt.cost_units_consumed,
        receipt.cost_units_consumed
    );
}
