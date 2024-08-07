use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_tests::common::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use scrypto_test::prelude::*;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;

fn get_ledger() -> (
    LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    ComponentAddress,
) {
    let (code, definition) = Compile::compile(
        path_local_blueprint!("system_wasm_buffers"),
        CompileProfile::FastWithTraceLogs,
    );

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);
    let component_address = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(package_address, "WasmBuffersTest", "new", manifest_args!())
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    (ledger, component_address)
}

macro_rules! test_wasm_buffer_read_write {
    ($ledger:expr, $component_address: expr, read=($buffer_size:expr, $memory_offs:expr, $memory_len:expr), write=($write_buffer_size:expr, $write_memory_offs:expr)) => {{
        let manifest_builder = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                $component_address,
                "read_memory",
                manifest_args!(
                    ($buffer_size) as usize,
                    ($memory_offs) as isize,
                    ($memory_len) as usize,
                ),
            )
            .call_method(
                $component_address,
                "write_memory",
                manifest_args!(($write_buffer_size) as usize, ($write_memory_offs) as isize,),
            );

        let manifest = manifest_builder.build();
        $ledger.execute_manifest(manifest, vec![])
    }};
}

macro_rules! test_wasm_buffer_read {
    ($ledger:expr, $component_address: expr, read=($buffer_size:expr, $memory_offs:expr, $memory_len:expr)) => {{
        let manifest_builder = ManifestBuilder::new().lock_fee_from_faucet().call_method(
            $component_address,
            "read_memory",
            manifest_args!(
                ($buffer_size) as usize,
                ($memory_offs) as isize,
                ($memory_len) as usize,
            ),
        );
        let manifest = manifest_builder.build();
        $ledger.execute_manifest(manifest, vec![])
    }};
}

macro_rules! test_wasm_buffer_consume {
    ($ledger:expr, $component_address: expr, buffer_id=$buffer_id:expr) => {{
        let manifest_builder = ManifestBuilder::new().lock_fee_from_faucet().call_method(
            $component_address,
            "write_memory_specific_buffer_id",
            manifest_args!($buffer_id as u32),
        );
        let manifest = manifest_builder.build();
        $ledger.execute_manifest(manifest, vec![])
    }};
    ($ledger:expr, $component_address: expr, buffer_ptr=$buffer_ptr:expr) => {{
        let manifest_builder = ManifestBuilder::new().lock_fee_from_faucet().call_method(
            $component_address,
            "write_memory_specific_buffer_ptr",
            manifest_args!($buffer_ptr as u32),
        );
        let manifest = manifest_builder.build();
        $ledger.execute_manifest(manifest, vec![])
    }};
}

fn get_sbor_len(buffer_size: u64) -> u64 {
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
fn test_wasm_buffer_read_write_memory_size_success() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();

    for buffer_size in [
        0u64,
        10 * KB,
        128 * KB,
        1 * MB,
        (MAX_SUBSTATE_VALUE_SIZE - 17) as u64, // maximum value possible to read and write
    ] {
        // Act
        let receipt = test_wasm_buffer_read_write!(
            ledger,
            component_address,
            read = (buffer_size, 0, get_sbor_len(buffer_size)),
            write = (buffer_size, 0)
        );

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn test_wasm_buffer_read_memory_access_error() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();

    for (buffer_size, memory_offs, memory_len) in [
        // Add 128KB to memory offs or memory len to make sure we are accessing beyond WASM memory

        // Small buffers
        (10 * KB, 0, 10 * KB + 128 * KB),
        (10 * KB, 64 * KB, 74 * KB),
        (10 * KB, 10 * KB + 127 * KB, 1 * KB),
        // Large buffers
        (1 * MB, 0, 1 * MB + 128 * KB),
    ] {
        // Act
        let receipt = test_wasm_buffer_read!(
            ledger,
            component_address,
            read = (buffer_size, memory_offs, memory_len)
        );

        // Assert
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
            )
        });
    }
}

#[test]
fn test_wasm_buffers_write_memory_access_error() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();

    for ((buffer_size, memory_offs, memory_len), (write_buffer_size, write_memory_offs)) in [
        // Add 128KB to write memory offs to make sure we are accessing beyond WASM memory
        (
            (10 * KB, 0, get_sbor_len(10 * KB)),
            (10 * KB, 10 * KB + 128 * KB),
        ),
        (
            (1 * MB, 0, get_sbor_len(1 * MB)),
            (1 * MB, 1 * MB + 128 * KB),
        ),
    ] {
        // Act
        let receipt = test_wasm_buffer_read_write!(
            ledger,
            component_address,
            read = (buffer_size, memory_offs, memory_len),
            write = (write_buffer_size, write_memory_offs)
        );

        // Assert
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
            )
        });
    }
}

#[test]
fn test_wasm_buffer_read_memory_substate_size_exceeded() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();

    // Act
    let receipt = test_wasm_buffer_read!(
        ledger,
        component_address,
        read = (
            MAX_SUBSTATE_VALUE_SIZE as u64,
            0,
            get_sbor_len(MAX_SUBSTATE_VALUE_SIZE as u64)
        )
    );

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
    let (mut ledger, component_address) = get_ledger();

    // Act
    let receipt = test_wasm_buffer_read!(
        ledger,
        component_address,
        read = (4 * MB, 0, get_sbor_len(4 * MB))
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        // This error is really nasty and we should somehow prevent it from occurring.
        // Especially that we know that transaction will fail for smaller sizes...
        RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::ExecutionError(message))) => {
            message == "Error { kind: TrapCode(UnreachableCodeReached) }"
        }
        _ => false,
    });

    // Act
    let receipt = test_wasm_buffer_read!(
        ledger,
        component_address,
        read = (
            256 * MB - 1, // SBOR max length
            0,
            get_sbor_len(256 * MB - 1)
        )
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::ExecutionError(message))) => {
            message == "Error { kind: TrapCode(UnreachableCodeReached) }"
        }
        _ => false,
    });
}

#[test]
fn test_wasm_buffer_read_memory_size_too_large() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();

    // Act
    let receipt = test_wasm_buffer_read!(
        ledger,
        component_address,
        read = (
            256 * MB, // SBOR max length exceeded
            0,
            get_sbor_len(256 * MB)
        )
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(message)) => {
            message.contains("SizeTooLarge")
        }
        _ => false,
    });

    // Act
    let receipt = test_wasm_buffer_read!(
        ledger,
        component_address,
        read = (usize::MAX as u64, 0, usize::MAX as u64)
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(message)) => {
            message.contains("SizeTooLarge")
        }
        _ => false,
    });
}

#[test]
fn test_wasm_buffer_invalid_buffer_id() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();

    for buffer_id in [0, 1, 3, u32::MAX] {
        // Act
        let receipt = test_wasm_buffer_consume!(ledger, component_address, buffer_id = buffer_id);
        // Assert
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::BufferNotFound(..))),
            )
        });
    }
    // Act
    let receipt = test_wasm_buffer_consume!(
        ledger,
        component_address,
        buffer_id = 2 // buffer_id=2 id of the kv_entry buffer
    );
    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_wasm_buffer_invalid_buffer_pointer() {
    // Arrange
    let (mut ledger, component_address) = get_ledger();
    // Write 1KB to KV store
    test_wasm_buffer_read!(
        ledger,
        component_address,
        read = (1 * KB, 0, get_sbor_len(1 * KB))
    );

    // Act
    let receipt = test_wasm_buffer_consume!(
        ledger,
        component_address,
        buffer_ptr = 0 // Invalid pointer
    );
    // Assert
    receipt.expect_commit_success(); // This is actually success because the WASM memory range is <0, pages_cnt * 64KB>

    // Act
    let receipt = test_wasm_buffer_consume!(ledger, component_address, buffer_ptr = u32::MAX);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::MemoryAccessError)),
        )
    });
}
