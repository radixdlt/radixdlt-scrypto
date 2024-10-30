use radix_common::prelude::*;
use radix_engine::errors::RejectionReason;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::updates::BabylonSettings;
use radix_engine_interface::blueprints::consensus_manager::EpochChangeCondition;
use scrypto_test::prelude::*;

use radix_transactions::validation::*;

#[test]
fn test_transaction_replay_protection() {
    let init_epoch = Epoch::of(1);
    let rounds_per_epoch = 5;
    let max_epoch_range = TransactionValidationConfig::latest().max_epoch_range;
    let genesis = BabylonSettings::test_default().with_consensus_manager_config(
        ConsensusManagerConfig::test_default().with_epoch_change_condition(EpochChangeCondition {
            min_round_count: rounds_per_epoch,
            max_round_count: rounds_per_epoch,
            target_duration_millis: 1000,
        }),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| genesis)
                .from_bootstrap_to_latest()
        })
        .build();

    // 1. Run a notarized transaction
    let transaction = create_notarized_transaction(TransactionParams {
        start_epoch_inclusive: init_epoch,
        end_epoch_exclusive: init_epoch.after(max_epoch_range).unwrap(),
    });

    let validator = TransactionValidator::new_with_latest_config(&NetworkDefinition::simulator());
    let validated = transaction
        .prepare_and_validate(&validator)
        .expect("Transaction should be validatable");
    let executable = validated.create_executable();

    let receipt = ledger.execute_transaction(
        executable.clone(),
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
    );
    receipt.expect_commit_success();

    // 2. Force update the epoch (through database layer)
    let new_epoch = init_epoch
        .after(max_epoch_range)
        .unwrap()
        .previous()
        .unwrap();
    ledger.set_current_epoch(new_epoch);

    // 3. Run the transaction again
    let receipt = ledger.execute_transaction(
        executable.clone(),
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
    );
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionReason::IntentHashPreviouslyCommitted(IntentHash::Transaction(_))
        )
    });

    // 4. Advance to the max epoch (which triggers epoch update)
    let receipt = ledger.advance_to_round(Round::of(rounds_per_epoch));

    // assert that precisely 1 partition was deleted:
    let partition_resets = receipt
        .expect_commit_success()
        .state_updates
        .by_node
        .values()
        .flat_map(|node_updates| match node_updates {
            NodeStateUpdates::Delta { by_partition } => by_partition.values(),
        })
        .filter_map(|partition_updates| match partition_updates {
            PartitionStateUpdates::Delta { .. } => None,
            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                new_substate_values,
            }) => Some(new_substate_values),
        })
        .collect::<Vec<_>>();
    // ... which means, there was 1x `BatchPartitionUpdate::Reset`...
    assert_eq!(partition_resets.len(), 1);
    // ... and it had empty new contents.
    assert!(partition_resets[0].is_empty());

    // 5. Run the transaction the 3rd time (with epoch range and intent hash checks disabled)
    // Note that in production, this won't be possible.
    let receipt = ledger.execute_transaction(
        executable
            .skip_epoch_range_check()
            .skip_intent_hash_nullification(),
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
    );
    receipt.expect_commit_success();
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
                .lock_fee_from_faucet()
                .drop_auth_zone_proofs()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
