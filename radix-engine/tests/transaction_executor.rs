use radix_engine::constants::*;
use radix_engine::engine::RuntimeError;
use radix_engine::engine::{ModuleError, RejectionError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmInstrumenter;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::{NotarizedTransaction, TransactionHeader, ValidatedTransaction};
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::{TestIntentHashManager, TransactionValidator, ValidationConfig};

#[test]
fn pre_execution_rejection_should_return_rejected_receipt() {
    // Arrange
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let executable_transaction = create_executable_transaction(1);

    // Act
    let receipt = test_runner.execute_transaction(
        &executable_transaction,
        &FeeReserveConfig::standard(),
        &ExecutionConfig::standard(),
    );

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
fn test_normal_transaction_flow() {
    // Arrange
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let intent_hash_manager = TestIntentHashManager::new();
    let validation_params = ValidationConfig {
        network_id: NetworkDefinition::simulator().id,
        current_epoch: 1,
        max_cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
        min_tip_percentage: 0,
    };
    let fee_reserve_config = FeeReserveConfig::standard();
    let execution_config = ExecutionConfig::debug();
    let raw_transaction = create_notarized_transaction(1_000_000).to_bytes();
    let validated_transaction = TransactionValidator::validate_from_slice(
        &raw_transaction,
        &intent_hash_manager,
        &validation_params,
    )
    .expect("Invalid transaction");
    let mut executor = TransactionExecutor::new(
        &mut substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
    );

    // Act
    let receipt = executor.execute_and_commit(
        &validated_transaction,
        &fee_reserve_config,
        &execution_config,
    );

    // Assert
    receipt.expect_commit_success();
}

fn create_executable_transaction(cost_unit_limit: u32) -> ValidatedTransaction {
    let notarized_transaction = create_notarized_transaction(cost_unit_limit);
    TransactionValidator::validate(
        notarized_transaction,
        &TestIntentHashManager::new(),
        &ValidationConfig {
            network_id: NetworkDefinition::simulator().id,
            current_epoch: 1,
            max_cost_unit_limit: 10_000_000,
            min_tip_percentage: 0,
        },
    )
    .unwrap()
}

fn create_notarized_transaction(cost_unit_limit: u32) -> NotarizedTransaction {
    // create key pairs
    let sk1 = EcdsaPrivateKey::from_u64(1).unwrap();
    let sk2 = EcdsaPrivateKey::from_u64(2).unwrap();
    let sk_notary = EcdsaPrivateKey::from_u64(3).unwrap();

    TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 5,
            notary_public_key: sk_notary.public_key().into(),
            notary_as_signatory: false,
            cost_unit_limit,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new(&NetworkDefinition::simulator())
                .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}
