use radix_engine::errors::{InterpreterError, RuntimeError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use ExpectedResult::{InvalidInput, InvalidOutput, Success};

#[test]
fn should_be_able_to_create_access_rules_with_undefined_method_name() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/package_schema");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "SchemaComponent",
            "create_component_with_access_rules_containing_undefined_method_name",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

enum ExpectedResult {
    Success,
    InvalidInput,
    InvalidOutput,
}

fn test_arg(method_name: &str, args: ManifestValue, expected_result: ExpectedResult) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/package_schema");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "SchemaComponent2", method_name, args)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    match expected_result {
        Success => {
            receipt.expect_commit_success();
        }
        InvalidInput => {
            receipt.expect_specific_failure(|e| {
                matches!(
                    e,
                    RuntimeError::InterpreterError(
                        InterpreterError::ScryptoInputSchemaNotMatch { .. }
                    )
                )
            });
        }
        InvalidOutput => {
            receipt.expect_specific_failure(|e| {
                matches!(
                    e,
                    RuntimeError::InterpreterError(
                        InterpreterError::ScryptoOutputSchemaNotMatch { .. }
                    )
                )
            });
        }
    }
}

#[test]
fn test_invalid_output_fails() {
    test_arg("invalid_output", to_manifest_value(&()), InvalidOutput)
}

#[test]
fn test_input_arg_unit_succeeds() {
    test_arg("unit", to_manifest_value(&()), Success)
}

#[test]
fn test_invalid_input_arg_unit_fails() {
    test_arg("unit", to_manifest_value(&0u8), InvalidInput)
}

#[test]
fn test_input_arg_bool_succeeds() {
    test_arg("bool", to_manifest_value(&true), Success)
}

#[test]
fn test_invalid_input_arg_bool_fails() {
    test_arg("unit", to_manifest_value(&0u8), InvalidInput)
}

#[test]
fn test_input_arg_ivalue_succeeds() {
    test_arg("i8", to_manifest_value(&0i8), Success);
    test_arg("i16", to_manifest_value(&0i16), Success);
    test_arg("i32", to_manifest_value(&0i32), Success);
    test_arg("i64", to_manifest_value(&0i64), Success);
    test_arg("i128", to_manifest_value(&0i128), Success);
}

#[test]
fn test_input_arg_ivalue_fails() {
    test_arg("i8", to_manifest_value(&()), InvalidInput);
    test_arg("i16", to_manifest_value(&()), InvalidInput);
    test_arg("i32", to_manifest_value(&()), InvalidInput);
    test_arg("i64", to_manifest_value(&()), InvalidInput);
    test_arg("i128", to_manifest_value(&()), InvalidInput);
}

#[test]
fn test_input_arg_uvalue_succeeds() {
    test_arg("u8", to_manifest_value(&0u8), Success);
    test_arg("u16", to_manifest_value(&0u16), Success);
    test_arg("u32", to_manifest_value(&0u32), Success);
    test_arg("u64", to_manifest_value(&0u64), Success);
    test_arg("u128", to_manifest_value(&0u128), Success);
}

#[test]
fn test_input_arg_uvalue_fails() {
    test_arg("u8", to_manifest_value(&()), InvalidInput);
    test_arg("u16", to_manifest_value(&()), InvalidInput);
    test_arg("u32", to_manifest_value(&()), InvalidInput);
    test_arg("u64", to_manifest_value(&()), InvalidInput);
    test_arg("u128", to_manifest_value(&()), InvalidInput);
}

#[test]
fn test_input_arg_result_succeeds() {
    let okay: Result<(), ()> = Ok(());
    let error: Result<(), ()> = Err(());
    test_arg("result", to_manifest_value(&okay), Success);
    test_arg("result", to_manifest_value(&error), Success);
}

#[test]
fn test_invalid_input_arg_result_fails() {
    test_arg("result", to_manifest_value(&0u8), InvalidInput);
}

#[test]
fn test_input_arg_tree_map_succeeds() {
    let mut tree_map = BTreeMap::new();
    tree_map.insert((), ());
    test_arg("tree_map", to_manifest_value(&tree_map), Success);
}

#[test]
fn test_invalid_input_arg_tree_map_fails() {
    test_arg("tree_map", to_manifest_value(&0u8), InvalidInput);
}

#[test]
fn test_input_arg_hash_set_succeeds() {
    let mut hash_set = HashSet::new();
    hash_set.insert(());
    test_arg("hash_set", to_manifest_value(&hash_set), Success);
}

#[test]
fn test_invalid_input_arg_hash_set_fails() {
    test_arg("hash_set", to_manifest_value(&0u8), InvalidInput);
}

macro_rules! to_and_from_manifest_value {
    ($v:ident) => {{
        let m: ManifestValue = to_manifest_value(&$v);
        from_manifest_value(&m).unwrap()
    }};
}

#[test]
fn test_to_from_manifest_value() {
    let v = 0u8;

    let from: u8 = to_and_from_manifest_value!(v);
    assert_eq!(v, from);

    let mut hash_set = HashSet::new();
    hash_set.insert(vec![0u8, 3u8]);
    let from: HashSet<Vec<u8>> = to_and_from_manifest_value!(hash_set);
    assert_eq!(hash_set, from);

    let mut tree_map = BTreeMap::new();
    tree_map.insert(-1i8, vec![1u8]);
    let from: BTreeMap<i8, Vec<u8>> = to_and_from_manifest_value!(tree_map);
    assert_eq!(tree_map, from);
}
