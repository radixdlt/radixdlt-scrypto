use radix_engine::{
    errors::{RejectionError, RuntimeError, SystemModuleError},
    system::system_modules::limits::TransactionLimitsError,
    transaction::{ExecutionConfig, FeeReserveConfig},
    types::*,
    vm::wasm::WASM_MEMORY_PAGE_SIZE,
};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::{builder::ManifestBuilder, model::TestTransaction};

#[test]
fn transaction_limit_call_frame_memory_exceeded() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Grow memory (wasm pages) to exceed default max wasm memory per instance.
    let grow_value: usize = DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME / WASM_MEMORY_PAGE_SIZE as usize;

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert, exceeded memory should be larger by 1 memory page than the limit
    let expected_mem = DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME + WASM_MEMORY_PAGE_SIZE as usize;
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxWasmInstanceMemoryExceeded(x),
        )) => *x == expected_mem,
        _ => false,
    })
}

#[test]
fn transaction_limit_memory_exceeded() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");

    // Calculate value of additional bytes to allocate per call to exceed
    // max wasm memory per transaction limit in nested calls.
    let grow_value: usize = DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME / 2;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(
            package_address,
            "TransactionLimitTest",
            "recursive_with_memory",
            manifest_args!(DEFAULT_MAX_CALL_DEPTH as u32, grow_value),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert

    // One call frame mem:
    //  => 18 pages from system
    //  => grow value pages from execution of blueprint
    //  => one aditional page from blueprint execution
    let call_frame_mem =
        (18 + grow_value / WASM_MEMORY_PAGE_SIZE as usize + 1) * WASM_MEMORY_PAGE_SIZE as usize;

    // Expected memory equals how many call_frame_mem can fit in per transaction
    // memory plus one, as the limit needs to be exceeded to break transaction.
    let expected_mem = (DEFAULT_MAX_WASM_MEM_PER_TRANSACTION / call_frame_mem + 1) * call_frame_mem;

    // If this assert fails, then adjust grow_value variable.
    assert!((DEFAULT_MAX_WASM_MEM_PER_TRANSACTION / call_frame_mem + 1) < DEFAULT_MAX_CALL_DEPTH);

    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxWasmMemoryExceeded(x),
        )) => *x == expected_mem,
        _ => false,
    })
}

#[test]
fn transaction_limit_exceeded_substate_read_count_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(
            package_address,
            "TransactionLimitTest",
            "read_kv_stores",
            manifest_args!(200 as u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = FeeReserveConfig::default();
    let mut execution_config = ExecutionConfig::for_test_transaction();
    // lower substate reads limit to avoid Fee limit transaction result
    execution_config.max_substate_reads_per_transaction = 150;
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
                TransactionLimitsError::MaxSubstateReadCountExceeded
            ))
        )
    });
}

#[test]
fn transaction_limit_exceeded_substate_write_count_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(
            package_address,
            "TransactionLimitTest",
            "write_kv_stores",
            manifest_args!(100 as u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = FeeReserveConfig::default();
    let mut execution_config = ExecutionConfig::for_test_transaction();
    // lower substate writes limit to avoid Fee limit transaction result
    execution_config.max_substate_writes_per_transaction = 100;
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
                TransactionLimitsError::MaxSubstateWriteCountExceeded
            ))
        )
    });
}

#[test]
fn transaction_limit_exceeded_substate_read_size_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(
            package_address,
            "TransactionLimitTest",
            "read_kv_stores",
            manifest_args!(100u32),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = FeeReserveConfig::default();
    let mut execution_config = ExecutionConfig::for_test_transaction().with_kernel_trace(true);
    // Setting maximum substate size to small value to activate transaction limit
    execution_config.max_substate_size = 10;
    let receipt = test_runner.execute_transaction(
        prepared.get_executable(btreeset!()),
        fee_config,
        execution_config.clone(),
    );

    // Assert
    receipt.expect_specific_rejection(|e| match e {
        RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::SystemModuleError(
            SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxSubstateReadSizeExceeded(size),
            ),
        )) => *size > execution_config.max_substate_size,
        _ => false,
    });
}

#[test]
fn transaction_limit_exceeded_substate_write_size_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");

    const SIZE: usize = 5000;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_value",
            manifest_args!(SIZE),
        )
        .build();

    let transactions = TestTransaction::new_from_nonce(manifest, 10);
    let prepared = transactions.prepare().unwrap();
    let fee_config = FeeReserveConfig::default();
    let mut execution_config = ExecutionConfig::for_test_transaction().with_kernel_trace(true);
    execution_config.max_substate_size = SIZE + 8 /* SBOR prefix */ - 1 /* lower limit to trigger error */;
    let receipt = test_runner.execute_transaction(
        prepared.get_executable(btreeset!()),
        fee_config,
        execution_config,
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxSubstateWriteSizeExceeded(x),
        )) => *x == SIZE as usize + 13, /* SBOR prefix + Substate wrapper */
        _ => false,
    })
}

#[test]
fn transaction_limit_exceeded_invoke_input_size_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&format!(
        r#"
            (module
                (data (i32.const 0) "{}")
                (memory $0 64)
                (export "memory" (memory $0))
            )
        "#,
        "i".repeat(DEFAULT_MAX_INVOKE_INPUT_SIZE)
    ));
    assert!(code.len() > DEFAULT_MAX_INVOKE_INPUT_SIZE);
    let manifest = ManifestBuilder::new()
        .publish_package_advanced(
            code,
            PackageDefinition::default(),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::SystemModuleError(
                SystemModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxInvokePayloadSizeExceeded(_)
                )
            ))
        )
    })
}

#[test]
fn test_default_substate_size_limit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/transaction_limits");
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50.into())
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
        .lock_fee(test_runner.faucet_component(), 50.into())
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
        RuntimeError::SystemModuleError(SystemModuleError::LimitingError(
            LimitingError::MaxSubstateWriteSizeExceeded(_),
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
        .lock_fee(test_runner.faucet_component(), 50.into())
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
        .lock_fee(test_runner.faucet_component(), 50.into())
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
        RuntimeError::SystemModuleError(SystemModuleError::LimitingError(
            LimitingError::MaxInvokePayloadSizeExceeded(_),
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
        .lock_fee(test_runner.faucet_component(), 50.into())
        .call_function(
            package_address,
            "SborOverflow",
            "write_large_value",
            manifest_args!(),
        )
        .build();
    test_runner.execute_manifest(manifest, vec![]);
}
