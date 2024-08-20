use radix_common::prelude::*;
use radix_engine::errors::RejectionReason;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::updates::ProtocolBuilder;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::ScryptoVm;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::errors::TransactionValidationError;
use scrypto_test::prelude::*;

use radix_transactions::validation::*;

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
    let receipt = ledger.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

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
    let receipt = ledger.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    let rejection_error = receipt.expect_rejection();
    assert_eq!(
        rejection_error,
        &RejectionReason::TransactionEpochNoLongerValid {
            valid_until: Some(valid_until_epoch),
            current_epoch
        }
    );
}

#[test]
fn test_normal_transaction_flow() {
    // Arrange
    let scrypto_vm = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_validator_config: WasmValidatorConfigV1::new(),
    };
    let vm_init = VmInit::new(&scrypto_vm, NoExtension);

    let mut substate_db = InMemorySubstateDatabase::standard();
    ProtocolBuilder::for_simulator().from_bootstrap_to_latest().commit_each_protocol_update(&mut substate_db);

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

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());
    let validated = validator
        .validate_from_raw(&raw_transaction)
        .expect("Invalid transaction");
    let executable = validated.get_executable();
    assert_eq!(executable.payload_size(), 1023 * 1024 + 380);

    // Act
    let receipt = execute_and_commit_transaction(
        &mut substate_db,
        vm_init,
        &execution_config,
        executable,
    );

    // Assert
    receipt.expect_commit_success();
}

fn get_validated(
    transaction: &NotarizedTransactionV1,
) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    validator.validate(transaction.prepare().unwrap())
}
