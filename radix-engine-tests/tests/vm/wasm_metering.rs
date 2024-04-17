use radix_engine::{
    errors::{RuntimeError, VmError},
    vm::wasm::WasmRuntimeError,
};
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_loop() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let code = wat2wasm(&include_local_wasm_str!("loop.wat").replace("${n}", "1000"));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest_with_costing_params(
        manifest,
        vec![],
        CostingParameters::babylon_genesis().with_execution_cost_unit_limit(15_000_000),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_finish_before_system_loan_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let code = wat2wasm(&include_local_wasm_str!("loop.wat").replace("${n}", "1"));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let code = wat2wasm(&include_local_wasm_str!("loop.wat").replace("${n}", "2000000"));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest_with_costing_params(
        manifest,
        vec![],
        CostingParameters::babylon_genesis().with_execution_cost_unit_limit(15_000_000),
    );

    // Assert
    receipt.expect_specific_failure(is_costing_error)
}

#[test]
fn test_recursion() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_local_wasm_str!("recursion.wat").replace("${n}", "256"));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let code = wat2wasm(&include_local_wasm_str!("recursion.wat").replace("${n}", "257"));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_wasm_error)
}

#[test]
fn test_grow_memory_within_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Grow memory size by `MAX_MEMORY_SIZE_IN_PAGES - 1`.
    // Note that initial memory size is 1 page.
    let grow_value = MAX_MEMORY_SIZE_IN_PAGES - 1;

    // Act
    let code =
        wat2wasm(&include_local_wasm_str!("memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_grow_memory_beyond_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Grow memory size by `MAX_MEMORY_SIZE_IN_PAGES`.
    // Note that initial memory size is 1 page.
    let grow_value = MAX_MEMORY_SIZE_IN_PAGES;

    // Act
    let code =
        wat2wasm(&include_local_wasm_str!("memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::ExecutionError(e)))
            if e.contains("Unreachable") =>
        {
            true
        }
        _ => false,
    })
}

#[test]
fn test_grow_memory_by_more_than_65536() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Max allowed value is 0xffff
    let grow_value = 0x10000;

    // Act
    let code =
        wat2wasm(&include_local_wasm_str!("memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::VmError(VmError::Wasm(WasmRuntimeError::ExecutionError(e)))
            if e.contains("Unreachable") =>
        {
            true
        }
        _ => false,
    })
}
