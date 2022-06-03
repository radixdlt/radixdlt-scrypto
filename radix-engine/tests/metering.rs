#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::model::extract_package;
use radix_engine::{
    ledger::InMemorySubstateStore,
    wasm::{default_wasm_engine, InvokeError},
};
use scrypto::call_data;
use test_runner::wat2wasm;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaPrivateKey;

#[test]
fn test_loop() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package_address = test_runner.publish_package_with_code(code);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", call_data!(f()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_loop_out_of_tbd() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package_address = test_runner.publish_package_with_code(code);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", call_data!(f()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::OutOfTbd { .. })
}

#[test]
fn test_recursion() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "128"));
    let package_address = test_runner.publish_package_with_code(code);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", call_data!(f()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "129"));
    let package_address = test_runner.publish_package_with_code(code);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", call_data!(f()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::WasmError { .. })
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "99999"));
    let package_address = test_runner.publish_package_with_code(code);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", call_data!(f()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_grow_memory_out_of_tbd() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package_address = test_runner.publish_package_with_code(code);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", call_data!(f()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::OutOfTbd { .. })
}
