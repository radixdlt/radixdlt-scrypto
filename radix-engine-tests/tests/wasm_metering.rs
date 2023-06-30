use radix_engine::{
    errors::{RuntimeError, VmError},
    types::*,
    vm::wasm::{WasmRuntimeError, DEFAULT_MAX_MEMORY_SIZE_IN_PAGES},
};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "1000"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_with_cost_unit_limit(manifest, vec![], 15_000_000);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_finish_before_system_loan_limit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "1"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_with_cost_unit_limit(manifest, vec![], 15_000_000);

    // Assert
    receipt.expect_specific_failure(is_costing_error)
}

#[test]
fn test_recursion() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "256"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "257"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_wasm_error)
}

#[test]
fn test_grow_memory_within_limit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Grow memory size by `DEFAULT_MAX_MEMORY_SIZE_IN_PAGES - 1`.
    // Note that initial memory size is 1 page.
    let grow_value = DEFAULT_MAX_MEMORY_SIZE_IN_PAGES - 1;

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_grow_memory_beyond_limit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Grow memory size by `DEFAULT_MAX_MEMORY_SIZE_IN_PAGES`.
    // Note that initial memory size is 1 page.
    let grow_value = DEFAULT_MAX_MEMORY_SIZE_IN_PAGES;

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
    let mut test_runner = TestRunner::builder().build();

    // Max allowed value is 0xffff
    let grow_value = 0x10000;

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
