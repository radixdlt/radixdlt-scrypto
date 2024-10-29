use radix_common::crypto::Hash;
use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::system::system_modules::costing::SystemLoanFeeReserve;
use radix_engine::transaction::CostingParameters;
use radix_engine::vm::wasm::*;
use radix_engine::vm::wasm_runtime::NoOpWasmRuntime;
use radix_engine_interface::blueprints::package::CodeHash;
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use radix_transactions::model::TransactionCostingParameters;
use wabt::wat2wasm;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;

macro_rules! grow_memory {
    ($instance:expr, $runtime:expr, $len:expr) => {
        let pages = $len / (64 * KB);
        let result =
            $instance.invoke_export("Test_grow_memory", vec![Buffer(pages)], &mut $runtime);
        assert!(result.is_ok());
    };
}
macro_rules! read_memory_ok {
    ($instance:expr, $runtime:expr, $offs:expr, $len:expr) => {
        let result = $instance.invoke_export(
            "Test_read_memory",
            vec![Buffer($offs), Buffer($len)],
            &mut $runtime,
        );
        assert!(result.is_ok());
    };
}
macro_rules! read_memory_err {
    ($instance:expr, $runtime:expr, $offs:expr, $len:expr, $err:path) => {
        let result = $instance.invoke_export(
            "Test_read_memory",
            vec![Buffer($offs), Buffer($len)],
            &mut $runtime,
        );
        assert_matches!(result.unwrap_err(), InvokeError::SelfError($err));
    };
}
macro_rules! write_memory_ok {
    ($instance:expr, $runtime:expr, $offs:expr, $len:expr) => {
        let result = $instance.invoke_export(
            "Test_write_memory",
            vec![Buffer($offs), Buffer($len)],
            &mut $runtime,
        );
        assert!(result.is_ok());
    };
}
macro_rules! write_memory_err {
    ($instance:expr, $runtime:expr, $offs:expr, $len:expr, $err:path) => {
        let result = $instance.invoke_export(
            "Test_write_memory",
            vec![Buffer($offs), Buffer($len)],
            &mut $runtime,
        );
        assert_matches!(result.unwrap_err(), InvokeError::SelfError($err));
    };
}

#[test]
fn test_wasm_memory_grow_read_write() {
    // Arrange
    let code = wat2wasm(&include_local_wasm_str!("memory_boundaries.wat")).unwrap();
    let wasm_engine = DefaultWasmEngine::default();
    let mut instance = wasm_engine.instantiate(CodeHash(Hash([0u8; 32])), &code);

    let fee_reserve = SystemLoanFeeReserve::new(
        CostingParameters::babylon_genesis(),
        TransactionCostingParameters::default(),
        false,
    );
    let mut wasm_execution_units_consumed = 0;
    let mut runtime: Box<dyn WasmRuntime> = Box::new(NoOpWasmRuntime::new(
        fee_reserve,
        &mut wasm_execution_units_consumed,
    ));
    // Initially there is 64KB memory (1 page) available
    let initial_size = 64 * KB;
    let mut current_size = initial_size;

    // Act & Assert
    for size in [
        current_size,
        128 * KB,
        2 * MB,
        4 * MB, // this is RE memory limit (MAX_MEMORY_SIZE_IN_PAGES) limit
        5 * MB, // but it is not honored at this level
    ] {
        if size != initial_size {
            grow_memory!(instance, runtime, size - current_size);
            current_size = size;
        }
        write_memory_ok!(instance, runtime, 0, 64 * KB);
        read_memory_ok!(instance, runtime, 0, 64 * KB);

        write_memory_ok!(instance, runtime, 0, current_size);
        read_memory_ok!(instance, runtime, 0, current_size);

        write_memory_ok!(instance, runtime, current_size - 4 * KB, 2 * KB);
        read_memory_ok!(instance, runtime, current_size - 4 * KB, 2 * KB);

        write_memory_ok!(instance, runtime, current_size - 1, 1);
        read_memory_ok!(instance, runtime, current_size - 1, 1);

        // Access outside memory
        write_memory_err!(
            instance,
            runtime,
            0,
            current_size + 1,
            WasmRuntimeError::MemoryAccessError
        );
        read_memory_err!(
            instance,
            runtime,
            0,
            current_size + 1,
            WasmRuntimeError::MemoryAccessError
        );

        read_memory_err!(
            instance,
            runtime,
            current_size - 4 * KB,
            4 * KB + 1,
            WasmRuntimeError::MemoryAccessError
        );
        write_memory_err!(
            instance,
            runtime,
            current_size - 4 * KB,
            4 * KB + 1,
            WasmRuntimeError::MemoryAccessError
        );

        read_memory_err!(
            instance,
            runtime,
            current_size - 1,
            2,
            WasmRuntimeError::MemoryAccessError
        );
        write_memory_err!(
            instance,
            runtime,
            current_size - 1,
            2,
            WasmRuntimeError::MemoryAccessError
        );
    }
}

#[test]
fn test_wasm_memory_is_clean() {
    // Arrange
    let code = wat2wasm(&include_local_wasm_str!("memory_boundaries.wat")).unwrap();
    let wasm_engine = DefaultWasmEngine::default();
    let mut instance = wasm_engine.instantiate(CodeHash(Hash([0u8; 32])), &code);

    let fee_reserve = SystemLoanFeeReserve::new(
        CostingParameters::babylon_genesis(),
        TransactionCostingParameters::default(),
        false,
    );
    let mut wasm_execution_units_consumed = 0;
    let mut runtime: Box<dyn WasmRuntime> = Box::new(NoOpWasmRuntime::new(
        fee_reserve,
        &mut wasm_execution_units_consumed,
    ));

    // Initially there is 64KB memory (1 page) available
    let initial_size = 64 * KB;
    let mut current_size = initial_size;

    // Act & Assert
    for size in [
        current_size,
        128 * KB,
        2 * MB,
        4 * MB, // this is RE memory limit (MAX_MEMORY_SIZE_IN_PAGES) limit
        5 * MB, // but it is not honored at this level
    ] {
        if size != initial_size {
            grow_memory!(instance, runtime, size - current_size);
            // Clear the first byte, it was used to return clean flag in previous step
            write_memory_ok!(instance, runtime, 0, 1);
            current_size = size;
        }
        // Check if WASM memory is clear after the initialization or growing
        let result = instance
            .invoke_export("Test_check_memory_is_clean", vec![], &mut runtime)
            .unwrap();
        let clean = result[0];

        assert!(clean == 1);
    }
}
