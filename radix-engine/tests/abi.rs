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

fn test_arg(method_name: &str, arg: Vec<u8>, should_succeed: bool) {
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
            "AbiComponent2",
            method_name,
            arg,
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    if should_succeed {
        receipt.result.expect("Should be okay.");
    } else {
        let error = receipt.result.expect_err("Should be an error.");
        if !matches!(error, RuntimeError::InvalidMethodArgument { .. }) {
            panic!("Error should be InvalidMethodArgument but was {:?}", error)
        }
    }
}

#[test]
fn test_input_arg_unit_succeeds() {
    test_arg("unit", scrypto_encode(&()), true)
}

#[test]
fn test_invalid_input_arg_unit_fails() {
    test_arg("unit", scrypto_encode(&0u8), false)
}

#[test]
fn test_input_arg_bool_succeeds() {
    test_arg("bool", scrypto_encode(&true), true)
}

#[test]
fn test_invalid_input_arg_bool_fails() {
    test_arg("unit", scrypto_encode(&0u8), false)
}

#[test]
fn test_input_arg_ivalue_succeeds() {
    test_arg("i8", scrypto_encode(&0i8), true);
    test_arg("i16", scrypto_encode(&0i16), true);
    test_arg("i32", scrypto_encode(&0i32), true);
    test_arg("i64", scrypto_encode(&0i64), true);
    test_arg("i128", scrypto_encode(&0i128), true);
}

#[test]
fn test_input_arg_ivalue_fails() {
    test_arg("i8", scrypto_encode(&()), false);
    test_arg("i16", scrypto_encode(&()), false);
    test_arg("i32", scrypto_encode(&()), false);
    test_arg("i64", scrypto_encode(&()), false);
    test_arg("i128", scrypto_encode(&()), false);
}

#[test]
fn test_input_arg_uvalue_succeeds() {
    test_arg("u8", scrypto_encode(&0u8), true);
    test_arg("u16", scrypto_encode(&0u16), true);
    test_arg("u32", scrypto_encode(&0u32), true);
    test_arg("u64", scrypto_encode(&0u64), true);
    test_arg("u128", scrypto_encode(&0u128), true);
}

#[test]
fn test_input_arg_uvalue_fails() {
    test_arg("u8", scrypto_encode(&()), false);
    test_arg("u16", scrypto_encode(&()), false);
    test_arg("u32", scrypto_encode(&()), false);
    test_arg("u64", scrypto_encode(&()), false);
    test_arg("u128", scrypto_encode(&()), false);
}

#[test]
fn test_input_arg_result_succeeds() {
    let okay: Result<(), ()> =  Ok(());
    let error: Result<(), ()> =  Err(());
    test_arg("result", scrypto_encode(&okay), true);
    test_arg("result", scrypto_encode(&error), true);
}

#[test]
fn test_invalid_input_arg_result_fails() {
    test_arg("result", scrypto_encode(&0u8), false);
}

#[test]
fn test_input_arg_tree_map_succeeds() {
    let mut tree_map = BTreeMap::new();
    tree_map.insert((), ());
    test_arg("tree_map", scrypto_encode(&tree_map), true);
}

#[test]
fn test_invalid_input_arg_tree_map_fails() {
    test_arg("tree_map", scrypto_encode(&0u8), false);
}

#[test]
fn test_input_arg_hash_set_succeeds() {
    let mut hash_set = HashSet::new();
    hash_set.insert(());
    test_arg("hash_set", scrypto_encode(&hash_set), true);
}

#[test]
fn test_invalid_input_arg_hash_set_fails() {
    test_arg("hash_set", scrypto_encode(&0u8), false);
}