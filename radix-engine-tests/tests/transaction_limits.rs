use radix_engine::{
    errors::{RuntimeError, SystemModuleError},
    system::system_modules::limits::TransactionLimitsError,
    transaction::{ExecutionConfig, FeeReserveConfig},
    types::*,
};
use scrypto_unit::*;
use transaction::{builder::ManifestBuilder, model::TestTransaction};

#[test]
fn transaction_limit_exceeded_substate_read_count_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(
            component_address,
            "read_kv_stores",
            manifest_args!(200 as u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = FeeReserveConfig::default();
    let mut execution_config = ExecutionConfig::for_test_transaction();
    // lower substate reads limit to avoid Fee limit transaction result
    execution_config.max_number_of_substates_in_track = 150;
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
                TransactionLimitsError::TooManyEntriesInTrack
            ))
        )
    });
}

#[test]
fn transaction_limit_exceeded_substate_write_count_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(
            component_address,
            "write_kv_stores",
            manifest_args!(100 as u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = FeeReserveConfig::default();
    let mut execution_config = ExecutionConfig::for_test_transaction();
    // lower substate writes limit to avoid Fee limit transaction result
    execution_config.max_number_of_substates_in_track = 100;
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
                TransactionLimitsError::TooManyEntriesInTrack
            ))
        )
    });
}

#[test]
fn test_default_substate_size_limit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 5000u32.into())
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_value",
            manifest_args!(DEFAULT_MAX_SUBSTATE_SIZE - 14),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act #2
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_value",
            manifest_args!(DEFAULT_MAX_SUBSTATE_SIZE - 13),
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
    encoder.write_size(DEFAULT_MAX_INVOKE_INPUT_SIZE).unwrap();
    let overhead_len = overhead.len();
    let actor_len = PACKAGE_PACKAGE.as_ref().len() + "InvokeLimitsTest".len() + "callee".len();
    println!("{:?}", overhead_len);
    println!("{:?}", actor_len);

    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "InvokeLimitsTest",
            "call",
            manifest_args!(DEFAULT_MAX_INVOKE_INPUT_SIZE - actor_len - overhead_len),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act #2
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "InvokeLimitsTest",
            "call",
            manifest_args!(DEFAULT_MAX_INVOKE_INPUT_SIZE - actor_len - overhead_len + 1),
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

// FIXME: THIS CAUSES OVERFLOW. INVESTIGATE AND FIX IT!
#[test]
#[ignore]
fn reproduce_crash() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "SborOverflow",
            "write_large_value",
            manifest_args!(),
        )
        .build();
    test_runner.execute_manifest(manifest, vec![]);
}

#[test]
fn verify_log_size_limit() {
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_limits");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "emit_log_of_size",
            manifest_args!(DEFAULT_MAX_LOG_SIZE + 1),
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_limits");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "emit_event_of_size",
            manifest_args!(DEFAULT_MAX_EVENT_SIZE + 1),
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_limits");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "panic_of_size",
            manifest_args!(DEFAULT_MAX_PANIC_MESSAGE_SIZE + 1),
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
