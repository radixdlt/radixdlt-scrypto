use radix_engine::{
    errors::*,
    transaction::TransactionReceipt,
    types::*,
    vm::{wasm::WasmRuntimeError, NativeVmExtension, NoExtension},
};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::*;
use transaction::prelude::*;

const KB: usize = 1024;
const MB: usize = 1024 * KB;

struct ReadMemory {
    buffer_size: usize,
    memory_offs: usize,
    memory_len: usize,
}

struct WriteMemory {
    buffer_size: usize,
    memory_offs: usize,
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
            read_memory.buffer_size,
            read_memory.memory_offs,
            read_memory.memory_len
        ),
    );
    if let Some(write_memory) = write_memory {
        manifest_builder = manifest_builder.call_method(
            component_address,
            "write_memory",
            manifest_args!(write_memory.buffer_size, write_memory.memory_offs),
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

#[test]
fn test_wasm_buffers_small_success() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 10 * KB,
            memory_offs: 0usize,
            memory_len: 10 * KB + 5, // +5 for SBOR headers
        },
        Some(WriteMemory {
            buffer_size: 10 * KB,
            memory_offs: 0usize,
        }),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_wasm_buffers_large_success() {
    // Arrange
    let (mut test_runner, component_address) = get_test_runner();

    // Act
    let receipt = build_and_execute_manifest(
        &mut test_runner,
        component_address,
        ReadMemory {
            buffer_size: 1 * MB,
            memory_offs: 0usize,
            memory_len: 1 * MB + 6, // +6 for SBOR headers
        },
        Some(WriteMemory {
            buffer_size: 1 * MB,
            memory_offs: 0usize,
        }),
    );

    // Assert
    receipt.expect_commit_success();
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
            memory_offs: 0usize,
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
            memory_offs: 0usize,
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
            memory_offs: 0usize,
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
