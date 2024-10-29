use radix_common::prelude::*;
use radix_engine::errors::RejectionReason;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::updates::ProtocolBuilder;
use radix_engine::vm::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::validation::*;
use scrypto_test::prelude::*;

#[test]
fn transaction_executed_before_valid_returns_that_rejection_reason() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let current_epoch = Epoch::of(150);
    let valid_from_epoch = Epoch::of(151);
    let valid_until_epoch = Epoch::of(151);

    ledger.set_current_epoch(current_epoch);

    let transaction = create_notarized_transaction(
        TransactionParams {
            start_epoch_inclusive: valid_from_epoch,
            end_epoch_exclusive: valid_until_epoch.next().unwrap(),
        },
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .drop_auth_zone_proofs()
            .build(),
    );

    // Act
    let receipt = ledger.execute_transaction(transaction, ExecutionConfig::for_test_transaction());

    // Assert
    let rejection_error = receipt.expect_rejection();
    assert_eq!(
        rejection_error,
        &RejectionReason::TransactionEpochNotYetValid {
            valid_from: valid_from_epoch,
            current_epoch
        }
    );
}

fn create_v2_transaction(
    min_proposer_timestamp_inclusive: Option<Instant>,
    max_proposer_timestamp_exclusive: Option<Instant>,
) -> NotarizedTransactionV2 {
    // create key pairs
    let signer = Secp256k1PrivateKey::from_u64(1).unwrap();
    let notary = Secp256k1PrivateKey::from_u64(2).unwrap();

    TransactionV2Builder::new()
        .intent_header(IntentHeaderV2 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::of(0),
            end_epoch_exclusive: Epoch::of(100),
            min_proposer_timestamp_inclusive,
            max_proposer_timestamp_exclusive,
            intent_discriminator: 0,
        })
        .manifest_builder(|builder| builder.lock_fee_from_faucet())
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: notary.public_key().into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        })
        .sign(&signer)
        .notarize(&notary)
        .build_minimal()
}

#[test]
fn transaction_with_invalid_timestamp_range_should_be_rejected() {
    let epoch = 1;
    let round = 5;
    let proposer_timestamp_ms = 1_000_000;
    let mut ledger = LedgerSimulatorBuilder::new().build();
    ledger.set_current_epoch(Epoch::of(epoch));
    ledger.advance_to_round_at_timestamp(Round::of(round), proposer_timestamp_ms);

    let receipt = ledger.execute_transaction(
        create_v2_transaction(Some(Instant::new(proposer_timestamp_ms / 1000 + 1)), None),
        ExecutionConfig::for_test_transaction(),
    );
    assert_matches!(
        receipt.expect_rejection(),
        &RejectionReason::TransactionProposerTimestampNotYetValid { .. }
    );

    let receipt = ledger.execute_transaction(
        create_v2_transaction(None, Some(Instant::new(proposer_timestamp_ms / 1000))),
        ExecutionConfig::for_test_transaction(),
    );
    assert_matches!(
        receipt.expect_rejection(),
        &RejectionReason::TransactionProposerTimestampNoLongerValid { .. }
    );

    let receipt = ledger.execute_transaction(
        create_v2_transaction(
            Some(Instant::new(proposer_timestamp_ms / 1000 - 1)),
            Some(Instant::new(proposer_timestamp_ms / 1000 + 1)),
        ),
        ExecutionConfig::for_test_transaction(),
    );
    receipt.expect_commit_success();
}

#[test]
fn transaction_executed_after_valid_returns_that_rejection_reason() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let current_epoch = Epoch::of(157);
    let valid_from_epoch = Epoch::of(151);
    let valid_until_epoch = Epoch::of(154);

    ledger.set_current_epoch(current_epoch);

    let transaction = create_notarized_transaction(
        TransactionParams {
            start_epoch_inclusive: valid_from_epoch,
            end_epoch_exclusive: valid_until_epoch.next().unwrap(),
        },
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .drop_auth_zone_proofs()
            .build(),
    );

    // Act
    let receipt = ledger.execute_transaction(transaction, ExecutionConfig::for_test_transaction());

    // Assert
    let rejection_error = receipt.expect_rejection();
    assert_eq!(
        rejection_error,
        &RejectionReason::TransactionEpochNoLongerValid {
            valid_until: valid_until_epoch,
            current_epoch
        }
    );
}

#[test]
fn test_normal_transaction_flow() {
    // Arrange
    let vm_modules = VmModules::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    ProtocolBuilder::for_simulator()
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut substate_db);

    let execution_config = ExecutionConfig::for_test_transaction().with_kernel_trace(true);
    let raw_transaction = create_notarized_transaction(
        TransactionParams {
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(100),
        },
        {
            let mut builder = ManifestBuilder::new();
            builder.add_blob([123u8; 1023 * 1024].to_vec());
            builder
                .lock_fee_from_faucet()
                .drop_auth_zone_proofs()
                .build()
        },
    )
    .to_raw()
    .unwrap();

    let validator = TransactionValidator::new_for_latest_simulator();
    let executable = raw_transaction
        .into_executable(&validator)
        .expect("Invalid transaction");
    assert_eq!(executable.payload_size(), 1023 * 1024 + 380);

    // Act
    let receipt = execute_and_commit_transaction(
        &mut substate_db,
        &vm_modules,
        &execution_config,
        executable,
    );

    // Assert
    receipt.expect_commit_success();
}
