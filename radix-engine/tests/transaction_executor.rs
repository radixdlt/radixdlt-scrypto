use radix_engine::engine::{ModuleError, RejectionError};
use radix_engine::engine::{RuntimeError, ScryptoInterpreter};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, InstructionCostRules, WasmMeteringConfig};
use radix_engine_lib::core::NetworkDefinition;
use radix_engine_constants::DEFAULT_MAX_COST_UNIT_LIMIT;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::{
    Executable, NotarizedTransaction, TransactionHeader, DEFAULT_MAX_EPOCH_RANGE,
};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator, ValidationConfig,
};

#[test]
fn pre_execution_rejection_should_return_rejected_receipt() {
    // Arrange
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let transaction = create_notarized_transaction(TransactionParams {
        cost_unit_limit: 1,
        start_epoch_inclusive: 0,
        end_epoch_exclusive: 10,
    });

    // Act
    let receipt = test_runner.execute_transaction(&get_executable(&transaction));

    // Assert
    let rejection_error = receipt.expect_rejection();
    if !matches!(
        rejection_error,
        RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::ModuleError(
            ModuleError::CostingError(..)
        ))
    ) {
        panic!("Expected costing error but was {}", rejection_error);
    }
}

#[test]
fn transaction_executed_before_valid_returns_that_rejection_reason() {
    // Arrange
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);

    const CURRENT_EPOCH: u64 = 150;
    const VALID_FROM_EPOCH: u64 = 151;
    const VALID_UNTIL_EPOCH: u64 = 151;

    test_runner.set_current_epoch(CURRENT_EPOCH);

    let transaction = create_notarized_transaction(TransactionParams {
        cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
        start_epoch_inclusive: VALID_FROM_EPOCH,
        end_epoch_exclusive: VALID_UNTIL_EPOCH + 1,
    });

    // Act
    let receipt = test_runner.execute_transaction(&get_executable(&transaction));

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
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);

    const CURRENT_EPOCH: u64 = 157;
    const VALID_FROM_EPOCH: u64 = 151;
    const VALID_UNTIL_EPOCH: u64 = 154;

    test_runner.set_current_epoch(CURRENT_EPOCH);

    let transaction = create_notarized_transaction(TransactionParams {
        cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
        start_epoch_inclusive: VALID_FROM_EPOCH,
        end_epoch_exclusive: VALID_UNTIL_EPOCH + 1,
    });

    // Act
    let receipt = test_runner.execute_transaction(&get_executable(&transaction));

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
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();

    let mut scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::new(
            InstructionCostRules::tiered(1, 5, 10, 5000),
            512,
        ),
    };

    let intent_hash_manager = TestIntentHashManager::new();
    let fee_reserve_config = FeeReserveConfig::standard();
    let execution_config = ExecutionConfig::debug();
    let raw_transaction = create_notarized_transaction(TransactionParams {
        cost_unit_limit: 1_000_000,
        start_epoch_inclusive: 0,
        end_epoch_exclusive: 0 + DEFAULT_MAX_EPOCH_RANGE,
    })
    .to_bytes();

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());
    let transaction = validator
        .check_length_and_decode_from_slice(&raw_transaction)
        .expect("Invalid transaction");

    let executable = validator
        .validate(&transaction, &intent_hash_manager)
        .expect("Invalid transaction");

    // Act
    let receipt = execute_and_commit_transaction(
        &mut substate_store,
        &mut scrypto_interpreter,
        &fee_reserve_config,
        &execution_config,
        &executable,
    );

    // Assert
    receipt.expect_commit_success();
}

fn get_executable<'a>(transaction: &'a NotarizedTransaction) -> Executable<'a> {
    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    validator
        .validate(&transaction, &TestIntentHashManager::new())
        .unwrap()
}

struct TransactionParams {
    cost_unit_limit: u32,
    start_epoch_inclusive: u64,
    end_epoch_exclusive: u64,
}

fn create_notarized_transaction(params: TransactionParams) -> NotarizedTransaction {
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
            ManifestBuilder::new(&NetworkDefinition::simulator())
                .lock_fee(FAUCET_COMPONENT, 10.into())
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
