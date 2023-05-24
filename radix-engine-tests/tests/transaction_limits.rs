use radix_engine::{
    errors::{ModuleError, RejectionError, RuntimeError},
    system::system_modules::transaction_limits::TransactionLimitsError,
    transaction::{ExecutionConfig, FeeReserveConfig},
    types::*,
    vm::wasm::WASM_MEMORY_PAGE_SIZE,
};
use radix_engine_interface::schema::PackageSchema;
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
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRule::DenyAll,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert, exceeded memory should be larger by 1 memory page than the limit
    let expected_mem = DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME + WASM_MEMORY_PAGE_SIZE as usize;
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
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
        .lock_fee(test_runner.faucet_component(), 10.into())
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
        RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
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
        .lock_fee(test_runner.faucet_component(), 10.into())
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
    let mut execution_config = ExecutionConfig::default();
    // lower substate reads limit to avoid Fee limit transaction result
    execution_config.max_substate_reads_per_transaction = 150;
    let receipt = test_runner.execute_transaction_with_config(
        prepared.get_executable(btreeset!()),
        &fee_config,
        &execution_config,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
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
        .lock_fee(test_runner.faucet_component(), 10.into())
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
    let mut execution_config = ExecutionConfig::default();
    // lower substate writes limit to avoid Fee limit transaction result
    execution_config.max_substate_writes_per_transaction = 100;
    let receipt = test_runner.execute_transaction_with_config(
        prepared.get_executable(btreeset!()),
        &fee_config,
        &execution_config,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
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
        .lock_fee(test_runner.faucet_component(), 10.into())
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
    let mut execution_config = ExecutionConfig::default();
    // Setting maximum substate size to small value to activate transaction limit
    execution_config.max_substate_size = 10;
    execution_config.kernel_trace = true;
    let receipt = test_runner.execute_transaction_with_config(
        prepared.get_executable(btreeset!()),
        &fee_config,
        &execution_config,
    );

    // Assert
    receipt.expect_specific_rejection(|e| match e {
        RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::ModuleError(
            ModuleError::TransactionLimitsError(
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

    const SIZE: u32 = 5000;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 100.into())
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
    let mut execution_config = ExecutionConfig::default();
    execution_config.max_substate_size = SIZE as usize + 8 /* SBOR prefix */ - 1 /* lower limit to trigger error */;
    execution_config.kernel_trace = true;
    let receipt = test_runner.execute_transaction_with_config(
        prepared.get_executable(btreeset!()),
        &fee_config,
        &execution_config,
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxSubstateWriteSizeExceeded(x),
        )) => *x == SIZE as usize + 8, /* SBOR prefix */
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
            PackageSchema::default(),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRule::DenyAll,
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxInvokePayloadSizeExceeded(_)
                )
            ))
        )
    })
}

#[test]
fn transaction_limit_exceeded_direct_invoke_input_size_should_fail() {
    // Arrange
    let data: Vec<u8> = (0..DEFAULT_MAX_INVOKE_INPUT_SIZE).map(|_| 0).collect();
    let blueprint_name = "test_blueprint";
    let function_name = "test_fn";
    let package_address = PACKAGE_PACKAGE;

    // Act
    let ret =
        TestRunner::kernel_invoke_function(package_address, blueprint_name, function_name, &data);

    // Assert
    let err = ret.expect_err("Expected failure but was success");

    let size = scrypto_args!(data).len()
        + blueprint_name.len()
        + function_name.len()
        + package_address.as_ref().len();

    match err {
        RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
            TransactionLimitsError::MaxInvokePayloadSizeExceeded(x),
        )) => assert_eq!(x, size),
        x => panic!(
            "Expected specific failure but was different error:\n{:?}",
            x
        ),
    }
}
