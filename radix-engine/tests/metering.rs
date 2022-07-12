#[rustfmt::skip]
pub mod test_runner;

use radix_engine::fee::{ENGINE_RUN_COST, TX_VALIDATION_COST_PER_BYTE, WASM_ENGINE_CALL_COST};
use radix_engine::wasm::InvokeError;
use scrypto::prelude::Package;
use scrypto::to_struct;
use test_runner::{abi_single_fn_any_input_void_output, wat2wasm, TestRunner};
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::CostingError { .. })
}

#[test]
fn test_recursion() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "128"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "129"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::WasmError { .. })
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_grow_memory_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::CostingError { .. })
}

#[test]
fn test_total_cost_units_consumed() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/syscall.wat"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    /*
    Cost analysis:
    1. Transaction validation cost = TX_VALIDATION_COST_PER_BYTE * 1
    2. Engine run cost = ENGINE_RUN_COST * 4
       * TransactionProcessor::main
          * Scrypto::main
          * AuthZone::clear * 2
       * AuthZone::clear
    3. Wasm run cost = WASM_ENGINE_CALL_COST + 307
    */
    assert_eq!(
        TX_VALIDATION_COST_PER_BYTE * 1 + ENGINE_RUN_COST * 4 + WASM_ENGINE_CALL_COST + 307,
        receipt.cost_units_consumed
    );
}
