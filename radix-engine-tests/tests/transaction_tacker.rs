use radix_engine::errors::RejectionError;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine_interface::blueprints::consensus_manager::EpochChangeCondition;
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
    let init_epoch = Epoch::of(1);
    let rounds_per_epoch = 5;
    let genesis = CustomGenesis::default(
        init_epoch,
        CustomGenesis::default_consensus_manager_config().with_epoch_change_condition(
            EpochChangeCondition {
                min_round_count: rounds_per_epoch,
                max_round_count: rounds_per_epoch,
                target_duration_millis: 1000,
            },
        ),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // 1. Run a notarized transaction
    let transaction = create_notarized_transaction(TransactionParams {
        start_epoch_inclusive: init_epoch,
        end_epoch_exclusive: init_epoch.after(DEFAULT_MAX_EPOCH_RANGE),
    });
    let validated = get_validated(&transaction).unwrap();
    let receipt = test_runner.execute_transaction(
        validated.get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    receipt.expect_commit_success();

    // 2. Force update the epoch (through database layer)
    let new_epoch = init_epoch.after(DEFAULT_MAX_EPOCH_RANGE).previous();
    test_runner.set_current_epoch(new_epoch);

    // 3. Run the transaction again
    let receipt = test_runner.execute_transaction(
        validated.get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    receipt.expect_specific_rejection(|e| match e {
        RejectionError::IntentHashPreviouslyCommitted => true,
        _ => false,
    });

    // 4. Advance to the max epoch (which triggers epoch update)
    let receipt = test_runner.advance_to_round(Round::of(rounds_per_epoch));
    assert_eq!(
        receipt
            .expect_commit_success()
            .state_updates
            .partition_deletions
            .len(),
        1
    );

    // 5. Run the transaction the 3rd time (with epoch range check disabled)
    // Note that in production, this won't be possible.
    let mut executable = validated.get_executable();
    executable.skip_epoch_range_check();
    let receipt = test_runner.execute_transaction(
        executable,
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    receipt.expect_commit_success();
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
                .lock_fee(FAUCET, 500u32.into())
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
