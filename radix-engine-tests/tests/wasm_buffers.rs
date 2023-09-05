use radix_engine::errors::*;
use radix_engine::system::system_modules::costing::SystemLoanFeeReserve;
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine::transaction::{CostingParameters, TransactionReceipt};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::wasm_runtime::NoOpWasmRuntime;
use radix_engine::vm::*;
use radix_engine_common::crypto::Hash;
use radix_engine_interface::blueprints::package::CodeHash;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::*;
use transaction::model::TransactionCostingParameters;
use transaction::prelude::*;
use wabt::wat2wasm;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;

#[derive(Clone)]
struct ReadMemory {
    buffer_size: u64,
    memory_offs: u64,
    memory_len: u64,
}

#[derive(Clone)]
struct WriteMemory {
    buffer_size: u64,
    memory_offs: u64,
}

fn build_and_execute_manifest<E: NativeVmExtension, D: TestDatabase>(
    test_runner: &mut TestRunner<E, D>,
    component_address: ComponentAddress,
    read_memory: ReadMemory,
    write_memory: Option<WriteMemory>,
) -> TransactionReceipt {
    let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet().call_method(
        component_address,
        "read_memory",
        manifest_args!(
            read_memory.buffer_size as usize,
            read_memory.memory_offs as usize,
            read_memory.memory_len as usize
        ),
    );
    if let Some(write_memory) = write_memory {
        manifest_builder = manifest_builder.call_method(
            component_address,
            "write_memory",
            manifest_args!(
                write_memory.buffer_size as usize,
                write_memory.memory_offs as usize
            ),
        );
    }
    let manifest = manifest_builder.build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt
}

fn get_test_runner() -> (
    TestRunner<NoExtension, InMemorySubstateDatabase>,
    ComponentAddress,
) {
    let (code, definition) = Compile::compile("tests/blueprints/wasm_buffers");

    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address =
        test_runner.publish_package(code, definition, BTreeMap::new(), OwnerRole::None);
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(package_address, "WasmBuffersTest", "new", manifest_args!())
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    (test_runner, component_address)
}

fn get_memory_len(buffer_size: u64) -> u64 {
    if buffer_size == 0 {
        buffer_size + 4
    } else if buffer_size < 64 * KB {
        buffer_size + 5
    } else if buffer_size < 2 * MB {
        buffer_size + 6
    } else {
        buffer_size + 7
    }
}

#[test]
fn test_wasm_buffers_small_read_memory_access_error_1() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 10 * KB,
            memory_offs: 0,
            memory_len: 10 * KB + 128 * KB, // Add 128KB to make sure we are accessing beyond
                                            // WASM memory
        },
        None,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
        )
    });
}

#[test]
fn test_wasm_buffers_large_read_memory_access_error_1() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 1 * MB,
            memory_offs: 0,
            memory_len: 1 * MB + 128 * KB, // Add 128KB to make sure we are accessing not only
                                           // beyond the buffer but also whole WASM memory
        },
        None,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
        )
    });
}

#[test]
fn test_wasm_buffers_read_memory_access_error_2() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 10 * KB,
            memory_offs: 64 * KB, // memory offset + length is 128KB beyond the buffer and also
            memory_len: 74 * KB,  // beyond WASM memory
        },
        None,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
        )
    });
}

#[test]
fn test_wasm_buffers_read_memory_access_error_3() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 10 * KB,
            memory_offs: 10 * KB + 127 * KB, // memory offset + length is 128KB beyond the buffer and also
            memory_len: 1 * KB,
        },
        None,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
        )
    });
}

#[test]
fn test_wasm_buffers_write_memory_access_error_1() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 10 * KB,
            memory_offs: 0,
            memory_len: 10 * KB + 5, // +5 for SBOR headers
        },
        Some(WriteMemory {
            buffer_size: 10 * KB,
            memory_offs: 10 * KB + 128 * KB, // Add 128KB to make sure we are accessing not only
                                             // beyond the buffer but also whole WASM memory
        }),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
        )
    });
}

#[test]
fn test_wasm_buffer_read_write_memory_size_success() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    for buffer_size in [
        0u64,
        10 * KB,
        128 * KB,
        1 * MB,
        (MAX_SUBSTATE_VALUE_SIZE - 17) as u64, // maximum value possible to read and write
    ] {
        let read_memory = ReadMemory {
            buffer_size,
            memory_offs: 0,
            memory_len: get_memory_len(buffer_size),
        };
        let write_memory = WriteMemory {
            buffer_size,
            memory_offs: 0,
        };
        // Act
        let receipt = build_and_execute_manifest(
            &mut test_runner,
            component_address,
            read_memory,
            Some(write_memory),
        );

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn test_wasm_buffer_read_memory_substate_size_exceeded() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    let read_memory = ReadMemory {
        buffer_size: 2 * MB,
        memory_offs: 0,
        memory_len: get_memory_len(2 * MB),
    };

    // Act
    let receipt =
        build_and_execute_manifest(&mut test_runner, component_address, read_memory, None);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxSubstateSizeExceeded(..)
            ))
        )
    });
}

#[test]
fn test_wasm_buffer_read_memory_instruction_trap() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // This error is really nasty and we should somehow prevent it from occuring.
    // Especially that we know that transaction will fail for smaller sizes...
    let read_memory = ReadMemory {
        buffer_size: 4 * MB,
        memory_offs: 0,
        memory_len: get_memory_len(4 * MB),
    };

    // Act
    let receipt =
        build_and_execute_manifest(&mut test_runner, component_address, read_memory, None);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::ExecutionError(message))) => {
            message == "Trap(Trap { reason: InstructionTrap(UnreachableCodeReached) })"
        }
        _ => false,
    });

    let read_memory = ReadMemory {
        buffer_size: 256 * MB - 1,
        memory_offs: 0,
        memory_len: get_memory_len(256 * MB - 1),
    };

    // Act
    let receipt =
        build_and_execute_manifest(&mut test_runner, component_address, read_memory, None);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::ExecutionError(message))) => {
            message == "Trap(Trap { reason: InstructionTrap(UnreachableCodeReached) })"
        }
        _ => false,
    });
}

#[test]
fn test_wasm_buffer_read_memory_size_too_large() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    let read_memory = ReadMemory {
        buffer_size: 256 * MB, // Capacity of SBOR exceeded
        memory_offs: 0,
        memory_len: get_memory_len(256 * MB),
    };

    // Act
    let receipt =
        build_and_execute_manifest(&mut test_runner, component_address, read_memory, None);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(message)) => {
            message.contains("SizeTooLarge")
        }
        _ => false,
    });

    let read_memory = ReadMemory {
        buffer_size: usize::MAX as u64, // Capacity of SBOR exceeded
        memory_offs: 0,
        memory_len: usize::MAX as u64,
    };

    // Act
    let receipt =
        build_and_execute_manifest(&mut test_runner, component_address, read_memory, None);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(message)) => {
            message.contains("SizeTooLarge")
        }
        _ => false,
    });
}

#[test]
fn test_wasm_memory_boundaries() {
    // Arrange
    let code = wat2wasm(&include_str!("wasm/memory_boundaries.wat")).unwrap();
    let wasm_engine = DefaultWasmEngine::default();
    let mut instance = wasm_engine.instantiate(CodeHash(Hash([0u8; 32])), &code);

    let fee_reserve = SystemLoanFeeReserve::new(
        &CostingParameters::default(),
        &TransactionCostingParameters::default(),
        false,
    );
    let mut wasm_execution_units_consumed = 0;
    let mut runtime: Box<dyn WasmRuntime> = Box::new(NoOpWasmRuntime::new(
        fee_reserve,
        &mut wasm_execution_units_consumed,
    ));
    macro_rules! grow_memory {
        ($len:expr) => {
            let pages = $len / (64 * KB);
            let result =
                instance.invoke_export("Test_grow_memory", vec![Buffer(pages)], &mut runtime);
            assert!(result.is_ok());
        };
    }
    macro_rules! read_memory_ok {
        ($offs:expr, $len:expr) => {
            let result = instance.invoke_export(
                "Test_read_memory",
                vec![Buffer($offs), Buffer($len)],
                &mut runtime,
            );
            assert!(result.is_ok());
        };
    }
    macro_rules! read_memory_err {
        ($offs:expr, $len:expr, $err:path) => {
            let result = instance.invoke_export(
                "Test_read_memory",
                vec![Buffer($offs), Buffer($len)],
                &mut runtime,
            );
            assert!(matches!(result.unwrap_err(), InvokeError::SelfError($err)));
        };
    }
    macro_rules! write_memory_ok {
        ($offs:expr, $len:expr) => {
            let result = instance.invoke_export(
                "Test_write_memory",
                vec![Buffer($offs), Buffer($len)],
                &mut runtime,
            );
            assert!(result.is_ok());
        };
    }
    macro_rules! write_memory_err {
        ($offs:expr, $len:expr, $err:path) => {
            let result = instance.invoke_export(
                "Test_write_memory",
                vec![Buffer($offs), Buffer($len)],
                &mut runtime,
            );
            assert!(matches!(result.unwrap_err(), InvokeError::SelfError($err)));
        };
    }

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
            grow_memory!(size - current_size);
            current_size = size;
        }
        write_memory_ok!(0, 64 * KB);
        read_memory_ok!(0, 64 * KB);

        write_memory_ok!(0, current_size);
        read_memory_ok!(0, current_size);

        write_memory_ok!(current_size - 4 * KB, 2 * KB);
        read_memory_ok!(current_size - 4 * KB, 2 * KB);

        write_memory_ok!(current_size - 1, 1);
        read_memory_ok!(current_size - 1, 1);

        // Access outside memory
        write_memory_err!(0, current_size + 1, WasmRuntimeError::MemoryAccessError);
        read_memory_err!(0, current_size + 1, WasmRuntimeError::MemoryAccessError);

        read_memory_err!(
            current_size - 4 * KB,
            4 * KB + 1,
            WasmRuntimeError::MemoryAccessError
        );
        write_memory_err!(
            current_size - 4 * KB,
            4 * KB + 1,
            WasmRuntimeError::MemoryAccessError
        );

        read_memory_err!(current_size - 1, 2, WasmRuntimeError::MemoryAccessError);
        write_memory_err!(current_size - 1, 2, WasmRuntimeError::MemoryAccessError);
    }
}
