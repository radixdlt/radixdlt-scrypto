#[rustfmt::skip]
pub mod test_runner;

use radix_engine::engine::RuntimeError;
use radix_engine::model::PackageError;
use radix_engine::wasm::*;
use sbor::Type;
use scrypto::abi::*;
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
    receipt.expect_err(|e| {
        matches!(
            e,
            &RuntimeError::PackageError(PackageError::InvalidWasm(PrepareError::InvalidMemory(
                InvalidMemory::NoMemorySection
            )))
        )
    });
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
    receipt.expect_err(|e| {
        if let RuntimeError::InvokeError(b) = e {
            matches!(**b, InvokeError::MemoryAccessError)
        } else {
            false
        }
    });
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
    receipt.expect_err(|e| {
        if let RuntimeError::InvokeError(b) = e {
            matches!(**b, InvokeError::MemoryAccessError)
        } else {
            false
        }
    });
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
    receipt.expect_err(|e| matches!(e, RuntimeError::InvokeError(_)));
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
    receipt.expect_success();
}

#[test]
fn test_basic_package_missing_export() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let mut blueprints = HashMap::new();
    blueprints.insert(
        "some_blueprint".to_string(),
        BlueprintAbi {
            structure: Type::Unit,
            fns: vec![Fn {
                ident: "f".to_string(),
                mutability: Option::None,
                input: Type::Unit,
                output: Type::Unit,
                export_name: "f".to_string(),
            }],
        },
    );

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let package = Package { code, blueprints };
    let manifest = ManifestBuilder::new().publish_package(package).build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| {
        matches!(
            e,
            RuntimeError::PackageError(PackageError::InvalidWasm(
                PrepareError::MissingExport { .. }
            ))
        )
    });
}
