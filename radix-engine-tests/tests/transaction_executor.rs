use radix_engine::errors::RejectionError;
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::ScryptoVm;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::errors::TransactionValidationError;
use transaction::model::{
    NotarizedTransactionV1, TransactionHeaderV1, TransactionPayload,
    ValidatedNotarizedTransactionV1,
};
use transaction::prelude::TransactionManifestV1;
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

#[test]
fn transaction_executed_before_valid_returns_that_rejection_reason() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    let current_epoch = Epoch::of(150);
    let valid_from_epoch = Epoch::of(151);
    let valid_until_epoch = Epoch::of(151);

    test_runner.set_current_epoch(current_epoch);

    let transaction = create_notarized_transaction(
        TransactionParams {
            start_epoch_inclusive: valid_from_epoch,
            end_epoch_exclusive: valid_until_epoch.next(),
        },
        ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .clear_auth_zone()
            .build(),
    );

    // Act
    let receipt = test_runner.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    let rejection_error = receipt.expect_rejection();
    assert_eq!(
        rejection_error,
        &RejectionError::TransactionEpochNotYetValid {
            valid_from: valid_from_epoch,
            current_epoch
        }
    );
}

#[test]
fn transaction_executed_after_valid_returns_that_rejection_reason() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    let current_epoch = Epoch::of(157);
    let valid_from_epoch = Epoch::of(151);
    let valid_until_epoch = Epoch::of(154);

    test_runner.set_current_epoch(current_epoch);

    let transaction = create_notarized_transaction(
        TransactionParams {
            start_epoch_inclusive: valid_from_epoch,
            end_epoch_exclusive: valid_until_epoch.next(),
        },
        ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .clear_auth_zone()
            .build(),
    );

    // Act
    let receipt = test_runner.execute_transaction(
        get_validated(&transaction).unwrap().get_executable(),
        FeeReserveConfig::default(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    let rejection_error = receipt.expect_rejection();
    assert_eq!(
        rejection_error,
        &RejectionError::TransactionEpochNoLongerValid {
            valid_until: valid_until_epoch,
            current_epoch
        }
    );
}

#[test]
fn test_normal_transaction_flow() {
    // Arrange
    let mut scrypto_interpreter = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_validator_config: WasmValidatorConfigV1::new(),
    };
    let mut substate_db = InMemorySubstateDatabase::standard();
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter, true)
        .bootstrap_test_default()
        .unwrap();

    let fee_reserve_config = FeeReserveConfig::default();
    let execution_config = ExecutionConfig::for_test_transaction().with_kernel_trace(true);
    let raw_transaction = create_notarized_transaction(
        TransactionParams {
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(100),
        },
        ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .add_blob([123u8; 1023 * 1024].to_vec())
            .clear_auth_zone()
            .build(),
    )
    .to_raw()
    .unwrap();

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());
    let validated = validator
        .validate_from_raw(&raw_transaction)
        .expect("Invalid transaction");
    let executable = validated.get_executable();
    assert_eq!(executable.payload_size(), 1023 * 1024 + 391);

    // Act
    let receipt = execute_and_commit_transaction(
        &mut substate_db,
        &mut scrypto_interpreter,
        &fee_reserve_config,
        &execution_config,
        &executable,
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

struct TransactionParams {
    start_epoch_inclusive: Epoch,
    end_epoch_exclusive: Epoch,
}

fn create_notarized_transaction(
    params: TransactionParams,
    manifest: TransactionManifestV1,
) -> NotarizedTransactionV1 {
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
        .manifest(manifest)
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
