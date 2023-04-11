use radix_engine::errors::RejectionError;
use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::system::bootstrap::bootstrap;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::errors::{HeaderValidationError, TransactionValidationError};
use transaction::model::{Executable, NotarizedTransaction, TransactionHeader};
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator, ValidationConfig,
};

#[test]
fn low_cost_unit_limit_should_result_in_rejection() {
    // Arrange
    let transaction = create_notarized_transaction(
        component_address(EntityType::GlobalGenericComponent, 1),
        TransactionParams {
            cost_unit_limit: 1,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 10,
        },
    );

    // Act
    let result = get_executable(&transaction);

    // Assert
    assert_eq!(
        result.expect_err("Should be an error"),
        TransactionValidationError::HeaderValidationError(
            HeaderValidationError::InvalidCostUnitLimit
        )
    );
}

#[test]
fn transaction_executed_before_valid_returns_that_rejection_reason() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    const CURRENT_EPOCH: u64 = 150;
    const VALID_FROM_EPOCH: u64 = 151;
    const VALID_UNTIL_EPOCH: u64 = 151;

    test_runner.set_current_epoch(CURRENT_EPOCH);

    let transaction = create_notarized_transaction(
        test_runner.faucet_component(),
        TransactionParams {
            cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
            start_epoch_inclusive: VALID_FROM_EPOCH,
            end_epoch_exclusive: VALID_UNTIL_EPOCH + 1,
        },
    );

    // Act
    let receipt = test_runner.execute_transaction(get_executable(&transaction).unwrap());

    // Assert
    let rejection_error = receipt.expect_rejection();
    if !matches!(
        rejection_error,
        RejectionError::TransactionEpochNotYetValid {
            valid_from: VALID_FROM_EPOCH,
            current_epoch: CURRENT_EPOCH
        }
    ) {
        panic!(
            "Expected TransactionEpochNotYetValid error but was {}",
            rejection_error
        );
    }
}

#[test]
fn transaction_executed_after_valid_returns_that_rejection_reason() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    const CURRENT_EPOCH: u64 = 157;
    const VALID_FROM_EPOCH: u64 = 151;
    const VALID_UNTIL_EPOCH: u64 = 154;

    test_runner.set_current_epoch(CURRENT_EPOCH);

    let transaction = create_notarized_transaction(
        test_runner.faucet_component(),
        TransactionParams {
            cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
            start_epoch_inclusive: VALID_FROM_EPOCH,
            end_epoch_exclusive: VALID_UNTIL_EPOCH + 1,
        },
    );

    // Act
    let receipt = test_runner.execute_transaction(get_executable(&transaction).unwrap());

    // Assert
    let rejection_error = receipt.expect_rejection();
    if !matches!(
        rejection_error,
        RejectionError::TransactionEpochNoLongerValid {
            valid_until: VALID_UNTIL_EPOCH,
            current_epoch: CURRENT_EPOCH
        }
    ) {
        panic!(
            "Expected TransactionEpochNoLongerValid error but was {}",
            rejection_error
        );
    }
}

#[test]
fn test_normal_transaction_flow() {
    // Arrange
    let mut scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };
    let mut substate_db = InMemorySubstateDatabase::standard();
    let receipt = bootstrap(&mut substate_db, &scrypto_interpreter).unwrap();
    let faucet_component = receipt
        .expect_commit_success()
        .new_component_addresses()
        .last()
        .cloned()
        .unwrap();

    let intent_hash_manager = TestIntentHashManager::new();
    let fee_reserve_config = FeeReserveConfig::default();
    let execution_config = ExecutionConfig::standard();
    let raw_transaction = create_notarized_transaction(
        faucet_component,
        TransactionParams {
            cost_unit_limit: 5_000_000,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
        },
    )
    .to_bytes()
    .unwrap();

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());
    let transaction = validator
        .check_length_and_decode_from_slice(&raw_transaction)
        .expect("Invalid transaction");

    let executable = validator
        .validate(&transaction, raw_transaction.len(), &intent_hash_manager)
        .expect("Invalid transaction");

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

fn get_executable<'a>(
    transaction: &'a NotarizedTransaction,
) -> Result<Executable<'a>, TransactionValidationError> {
    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    validator.validate(&transaction, 0, &TestIntentHashManager::new())
}

struct TransactionParams {
    cost_unit_limit: u32,
    start_epoch_inclusive: u64,
    end_epoch_exclusive: u64,
}

fn create_notarized_transaction(
    faucet: ComponentAddress,
    params: TransactionParams,
) -> NotarizedTransaction {
    // create key pairs
    let sk1 = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let sk2 = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();
    let sk_notary = EcdsaSecp256k1PrivateKey::from_u64(3).unwrap();

    TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: params.start_epoch_inclusive,
            end_epoch_exclusive: params.end_epoch_exclusive,
            nonce: 5,
            notary_public_key: sk_notary.public_key().into(),
            notary_as_signatory: false,
            cost_unit_limit: params.cost_unit_limit,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new()
                .lock_fee(faucet, 10.into())
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
