#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn test_invalid_access_rule_methods() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let package_address = test_runner.publish_package("abi");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "AbiComponent",
            "create_invalid_abi_component",
            to_struct!(),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    if !matches!(error, RuntimeError::BlueprintFunctionDoesNotExist(_)) {
        panic!(
            "Should be an function does not exist but error was {}",
            error
        );
    }
}
