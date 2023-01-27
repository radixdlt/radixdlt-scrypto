use radix_engine::engine::{ApplicationError, InterpreterError, KernelError, RuntimeError};
use radix_engine::model::PackageError;
use radix_engine::types::*;
use radix_engine::wasm::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn missing_memory_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(
        r#"
            (module
                (func (export "test") (result i32)
                    i32.const 1337
                )
            )
            "#,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package(
            code,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRules::new(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            &RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::InvalidMemory(
                    InvalidMemory::NoMemorySection
                ))
            ))
        )
    });
}

#[test]
fn large_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "LargeReturnSize", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::KernelError(KernelError::WasmRuntimeError(b)) = e {
            matches!(*b, WasmRuntimeError::MemoryAccessError)
        } else {
            false
        }
    });
}

#[test]
fn overflow_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "MaxReturnSize", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::KernelError(KernelError::WasmRuntimeError(b)) = e {
            matches!(*b, WasmRuntimeError::MemoryAccessError)
        } else {
            false
        }
    });
}

#[test]
fn zero_return_len_should_cause_data_validation_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "ZeroReturnSize", "f", args!())
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::InterpreterError(InterpreterError::InvalidScryptoReturn(..))
        )
    });
}

#[test]
fn test_basic_package() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package(
            code,
            generate_single_function_abi("Test", "f", Type::Any),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRules::new(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_basic_package_missing_export() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let mut blueprints = BTreeMap::new();
    blueprints.insert(
        "some_blueprint".to_string(),
        BlueprintAbi {
            structure: Type::Tuple {
                element_types: vec![],
            },
            fns: vec![Fn {
                ident: "f".to_string(),
                mutability: Option::None,
                input: Type::Tuple {
                    element_types: vec![],
                },
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "f".to_string(),
            }],
        },
    );

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package(
            code,
            blueprints,
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRules::new(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::MissingExport { .. })
            ))
        )
    });
}
