use radix_engine::{
    errors::{RuntimeError, SystemModuleError, VmError},
    system::system_modules::limits::TransactionLimitsError,
    transaction::{CostingParameters, ExecutionConfig},
    vm::wasm::WasmRuntimeError,
};
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_read_non_existent_entries_from_kv_store_exceeding_limit() {
    let (code, definition) = PackageLoader::get("transaction_limits");
    let code_len = code.len();
    let definition_len = scrypto_encode(&definition).unwrap().len();

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);
    let component_address = ledger
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

    let transaction = TestTransaction::new_v1_from_nonce(manifest, 10, btreeset!());

    let execution_config = {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        let fee_config =
            CostingParameters::babylon_genesis().with_execution_cost_unit_limit(1_000_000_000);
        let mut limit_parameters = LimitParameters::babylon_genesis();
        limit_parameters.max_track_substate_total_bytes = code_len * 2 + definition_len + 10 * 1024;
        execution_config.system_overrides = Some(SystemOverrides {
            limit_parameters: Some(limit_parameters),
            costing_parameters: Some(fee_config),
            ..Default::default()
        });
        execution_config
    };

    let receipt = ledger.execute_transaction(transaction, execution_config);

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
    let (code, definition) = PackageLoader::get("transaction_limits");
    let code_len = code.len();
    let definition_len = scrypto_encode(&definition).unwrap().len();

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);
    let component_address = ledger
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

    let transaction =
        TestTransaction::new_v1_from_nonce(manifest, 10, btreeset!()).into_executable_unwrap();
    let execution_config = {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        let mut limit_parameters = LimitParameters::babylon_genesis();
        limit_parameters.max_track_substate_total_bytes = code_len * 2 + definition_len + 10 * 1024;
        let fee_config =
            CostingParameters::babylon_genesis().with_execution_cost_unit_limit(1_000_000_000);
        execution_config.system_overrides = Some(SystemOverrides {
            limit_parameters: Some(limit_parameters),
            costing_parameters: Some(fee_config),
            ..Default::default()
        });

        execution_config
    };

    let receipt = ledger.execute_transaction(transaction, execution_config);

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
fn test_write_entries_to_heap_kv_store_exceeding_limit() {
    let (code, definition) = PackageLoader::get("transaction_limits");

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "write_entries_to_heap_kv_store",
            manifest_args!(64 * 1024 as u32),
        )
        .build();

    let transaction = TestTransaction::new_v1_from_nonce(manifest, 10, btreeset!());

    let execution_config = {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        let mut limit_parameters = LimitParameters::babylon_genesis();
        limit_parameters.max_heap_substate_total_bytes = 1024 * 1024;
        let fee_config =
            CostingParameters::babylon_genesis().with_execution_cost_unit_limit(1_000_000_000);
        execution_config.system_overrides = Some(SystemOverrides {
            limit_parameters: Some(limit_parameters),
            costing_parameters: Some(fee_config),
            ..Default::default()
        });
        execution_config
    };

    let receipt = ledger.execute_transaction(transaction, execution_config);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::HeapSubstateSizeExceeded { .. }
            ))
        )
    });
}

#[test]
fn test_default_substate_size_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("transaction_limits"));
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_values",
            manifest_args!([MAX_SUBSTATE_VALUE_SIZE - 17]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act #2
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_values",
            manifest_args!([MAX_SUBSTATE_VALUE_SIZE - 16]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let actor_len = PACKAGE_PACKAGE.as_bytes().len() + "InvokeLimitsTest".len() + "callee".len();
    println!("{:?}", overhead_len);
    println!("{:?}", actor_len);

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("transaction_limits"));
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
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("transaction_limits"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "emit_log_of_size",
            manifest_args!(MAX_LOG_SIZE + 1),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("transaction_limits"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "emit_event_of_size",
            manifest_args!(MAX_EVENT_SIZE + 1),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("transaction_limits"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionLimitTest",
            "panic_of_size",
            manifest_args!(MAX_PANIC_MESSAGE_SIZE + 1),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::PanicMessageSizeTooLarge { .. }
            ),)
        )
    })
}

#[test]
fn test_allocating_buffers_exceeding_limit() {
    let (code, definition) = PackageLoader::get("transaction_limits");

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);
    let component_address = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(package_address, "BufferLimit", "new", manifest_args!())
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
            "allocate_buffers",
            manifest_args!(100 as u32),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::TooManyBuffers))
        )
    });
}
