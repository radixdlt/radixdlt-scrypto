use radix_engine::{types::*, wasm::WASM_MEMORY_PAGE_SIZE};
use radix_engine_constants::DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME;
use radix_engine_interface::blueprints::resource::*;
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
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRulesConfig::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_with_cost_unit_limit(manifest, vec![], 15_000_000);

    // Assert
    receipt.expect_commit_success();
}

// TODO: investigate the case where cost_unit_limit < system_loan and transaction runs out of cost units.

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRulesConfig::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 450.into())
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
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRulesConfig::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
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
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRulesConfig::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_wasm_error)
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Calculate how much we can grow the memory (by wasm pages), subtract 1 to be below limit.
    let grow_value: usize =
        DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME / WASM_MEMORY_PAGE_SIZE as usize - 1;

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", &grow_value.to_string()));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRulesConfig::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_grow_memory_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package_address = test_runner.publish_package(
        code,
        single_function_package_schema("Test", "f"),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRulesConfig::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_costing_error)
}
