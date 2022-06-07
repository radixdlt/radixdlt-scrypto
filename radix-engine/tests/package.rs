#[rustfmt::skip]
pub mod test_runner;

use sbor::Type;
use scrypto::abi::{BlueprintAbi, Function};
use radix_engine::engine::RuntimeError;
use radix_engine::model::PackageError;
use radix_engine::wasm::InvokeError;
use radix_engine::wasm::PrepareError::NoMemory;
use radix_engine::wasm::PrepareError;
use scrypto::prelude::*;
use scrypto::to_struct;
use test_runner::{wat2wasm, TestRunner};
use transaction::builder::ManifestBuilder;

#[test]
fn missing_memory_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

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
    let package = Package {
        code,
        blueprints: HashMap::new(),
    };
    let manifest = ManifestBuilder::new().publish_package(package).build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be error.");
    assert_eq!(
        error,
        RuntimeError::PackageError(PackageError::InvalidWasm(NoMemory))
    );
}

#[test]
fn large_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package = test_runner.extract_and_publish_package("package");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package, "LargeReturnSize", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(
        error,
        RuntimeError::InvokeError(InvokeError::MemoryAccessError.into())
    );
}

#[test]
fn overflow_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package = test_runner.extract_and_publish_package("package");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package, "MaxReturnSize", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(
        error,
        RuntimeError::InvokeError(InvokeError::MemoryAccessError.into())
    );
}

#[test]
fn zero_return_len_should_cause_data_validation_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package = test_runner.extract_and_publish_package("package");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package, "ZeroReturnSize", "f", to_struct!())
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    if !matches!(error, RuntimeError::InvokeError(_)) {
        panic!("{} should be data validation error", error);
    }
}

#[test]
fn test_basic_package() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let package = Package {
        code,
        blueprints: HashMap::new(),
    };
    let manifest = ManifestBuilder::new().publish_package(package).build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_basic_package_missing_export() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let mut blueprints = HashMap::new();
    blueprints.insert("some_blueprint".to_string(), BlueprintAbi {
        value: Type::Unit,
        functions: vec![
            Function {
                name: "f".to_string(),
                mutability: Option::None,
                input: Type::Unit,
                output: Type::Unit,
                export_name: "f".to_string(),
            }
        ]
    });

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let package = Package {
        code,
        blueprints,
    };
    let manifest = ManifestBuilder::new().publish_package(package).build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert!(matches!(error, RuntimeError::PackageError(PackageError::InvalidWasm(PrepareError::MissingExport { .. }))))
}

