#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::PackageError;
use radix_engine::wasm::WasmValidationError::NoMemoryExport;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn missing_memory_should_cause_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);

    // Act
    let code: Vec<u8> = wabt::wat2wasm(
        r#"
            (module
                (func (export "test") (result i32)
                    i32.const 1337
                )
            )
            "#,
    )
    .expect("failed to parse wat");
    let transaction = test_runner
        .new_transaction_builder()
        .publish_package(&code)
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be error.");
    assert_eq!(
        error,
        RuntimeError::PackageError(PackageError::WasmValidationError(NoMemoryExport))
    );
}

#[test]
fn large_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let package = test_runner.publish_package("package");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(package, "LargeReturnSize", call_data!(something()))
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(error, RuntimeError::MemoryAccessError);
}

#[test]
fn overflow_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let package = test_runner.publish_package("package");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(package, "MaxReturnSize", call_data!(something()))
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(error, RuntimeError::MemoryAccessError);
}

#[test]
fn zero_return_len_should_cause_data_validation_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let package = test_runner.publish_package("package");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(package, "ZeroReturnSize", call_data!(something()))
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    if !matches!(error, RuntimeError::ParseScryptoValueError(_)) {
        panic!("{} should be data validation error", error);
    }
}
