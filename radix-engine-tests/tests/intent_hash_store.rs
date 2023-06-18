use radix_engine::blueprints::transaction_tracker::EPOCHS_PER_PARTITION;
use radix_engine::errors::RejectionError;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::errors::TransactionValidationError;
use transaction::model::{
    NotarizedTransactionV1, TransactionHeaderV1, TransactionPayload,
    ValidatedNotarizedTransactionV1,
};
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

#[test]
fn test_transaction_replay_protection() {
    let mut test_runner = TestRunner::builder().build();

    let current_epoch = Epoch::of(1);
    test_runner.set_current_epoch(current_epoch);
    let transaction = create_notarized_transaction(TransactionParams {
        start_epoch_inclusive: current_epoch,
        end_epoch_exclusive: current_epoch.after(DEFAULT_MAX_EPOCH_RANGE),
    });

    // 1. Run a notarized transaction
    let receipt = test_runner.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    receipt.expect_commit_success();

    // 2. Run the transaction again
    let receipt = test_runner.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    receipt.expect_specific_rejection(|e| match e {
        RejectionError::IntentHashCommitted => true,
        _ => false,
    });

    // 3. Update the epoch
    let new_epoch = current_epoch.after(EPOCHS_PER_PARTITION * 190);
    test_runner.set_current_epoch(new_epoch);

    // 4. Run another transaction
    let transaction = create_notarized_transaction(TransactionParams {
        start_epoch_inclusive: new_epoch,
        end_epoch_exclusive: new_epoch.after(1),
    });
    let receipt = test_runner.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    let result = receipt.expect_commit_success();
    assert_eq!(result.partition_deletions.len(), 1);
}

fn get_validated(
    transaction: &NotarizedTransactionV1,
) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    validator.validate(transaction.prepare().unwrap())
}

struct TransactionParams {
    start_epoch_inclusive: Epoch,
    end_epoch_exclusive: Epoch,
}

fn create_notarized_transaction(params: TransactionParams) -> NotarizedTransactionV1 {
    // create key pairs
    let sk1 = Secp256k1PrivateKey::from_u64(1).unwrap();
    let sk2 = Secp256k1PrivateKey::from_u64(2).unwrap();
    let sk_notary = Secp256k1PrivateKey::from_u64(3).unwrap();

    TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: params.start_epoch_inclusive,
            end_epoch_exclusive: params.end_epoch_exclusive,
            nonce: 5,
            notary_public_key: sk_notary.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new()
                .lock_fee(FAUCET, 10.into())
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
