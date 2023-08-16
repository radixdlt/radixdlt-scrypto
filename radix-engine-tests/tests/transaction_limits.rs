use radix_engine::{
    errors::{RuntimeError, SystemModuleError},
    system::system_modules::limits::TransactionLimitsError,
    transaction::{CostingParameters, ExecutionConfig},
    types::*,
};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_read_non_existent_entries_from_kv_store_exceeding_limit() {
    let (code, definition) = Compile::compile("tests/blueprints/transaction_limits");
    let code_len = code.len();
    let definition_len = scrypto_encode(&definition).unwrap().len();

    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address =
        test_runner.publish_package(code, definition, BTreeMap::new(), OwnerRole::None);
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    package_address,
                    "TransactionLimitTest",
                    "new",
                    manifest_args!(),
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "read_non_existent_entries_from_kv_store",
            manifest_args!(64 * 1024 as u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = CostingParameters::default().with_execution_cost_unit_limit(1_000_000_000);
    let mut execution_config = ExecutionConfig::for_test_transaction();
    execution_config.max_track_substates_total_bytes = code_len * 2 + definition_len + 10 * 1024;
    let receipt = test_runner.execute_transaction(
        prepared.get_executable(btreeset!()),
        fee_config,
        execution_config,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::TrackSubstateSizeExceeded { .. }
            ))
        )
    });
}

#[test]
fn test_write_entries_to_kv_store_exceeding_limit() {
    let (code, definition) = Compile::compile("tests/blueprints/transaction_limits");
    let code_len = code.len();
    let definition_len = scrypto_encode(&definition).unwrap().len();

    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address =
        test_runner.publish_package(code, definition, BTreeMap::new(), OwnerRole::None);
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    package_address,
                    "TransactionLimitTest",
                    "new",
                    manifest_args!(),
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "write_entries_to_kv_store",
            manifest_args!(64 * 1024 as u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = CostingParameters::default().with_execution_cost_unit_limit(1_000_000_000);
    let mut execution_config = ExecutionConfig::for_test_transaction();
    execution_config.max_track_substates_total_bytes = code_len * 2 + definition_len + 10 * 1024;
    let receipt = test_runner.execute_transaction(
        prepared.get_executable(btreeset!()),
        fee_config,
        execution_config,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::TrackSubstateSizeExceeded { .. }
            ))
        )
    });
}

#[test]
fn test_default_substate_size_limit() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_value",
            manifest_args!(MAX_SUBSTATE_VALUE_SIZE - 14),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act #2
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_value",
            manifest_args!(MAX_SUBSTATE_VALUE_SIZE - 13),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert #2
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxSubstateSizeExceeded(_),
        )) => true,
        _ => false,
    })
}

#[test]
fn test_default_invoke_payload_size_limit() {
    let mut overhead = Vec::new();
    let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut overhead, 100);
    encoder
        .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
        .unwrap();
    encoder.write_value_kind(ValueKind::Tuple).unwrap();
    encoder.write_size(1).unwrap();
    encoder.write_value_kind(ValueKind::Array).unwrap();
    encoder.write_value_kind(ValueKind::U8).unwrap();
    encoder.write_size(MAX_INVOKE_PAYLOAD_SIZE).unwrap();
    let overhead_len = overhead.len();
    let actor_len = PACKAGE_PACKAGE.as_ref().len() + "InvokeLimitsTest".len() + "callee".len();
    println!("{:?}", overhead_len);
    println!("{:?}", actor_len);

    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "InvokeLimitsTest",
            "call",
            manifest_args!(MAX_INVOKE_PAYLOAD_SIZE - actor_len - overhead_len),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act #2
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "InvokeLimitsTest",
            "call",
            manifest_args!(MAX_INVOKE_PAYLOAD_SIZE - actor_len - overhead_len + 1),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert #2
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxInvokePayloadSizeExceeded(_),
        )) => true,
        _ => false,
    })
}

#[test]
fn verify_log_size_limit() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_limits");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "emit_log_of_size",
            manifest_args!(MAX_LOG_SIZE + 1),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::LogSizeTooLarge { .. }
            ),)
        )
    })
}

#[test]
fn verify_event_size_limit() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_limits");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "emit_event_of_size",
            manifest_args!(MAX_EVENT_SIZE + 1),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::EventSizeTooLarge { .. }
            ),)
        )
    })
}

#[test]
fn verify_panic_size_limit() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_limits");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "panic_of_size",
            manifest_args!(MAX_PANIC_MESSAGE_SIZE + 1),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::PanicMessageSizeTooLarge { .. }
            ),)
        )
    })
}
