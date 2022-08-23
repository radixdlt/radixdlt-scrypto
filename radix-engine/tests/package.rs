use radix_engine::engine::{ApplicationError, KernelError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::PackageError;
use radix_engine::wasm::*;
use sbor::Type;
use scrypto::abi::*;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_publish_package_from_scrypto() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package = test_runner.extract_and_publish_package("package");

    let manifest1 = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package, "PackageTest", "publish", args!())
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_success();
}

#[test]
fn missing_memory_should_cause_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .publish_package(package)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package = test_runner.extract_and_publish_package("package");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package, "LargeReturnSize", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        if let RuntimeError::KernelError(KernelError::InvokeError(b)) = e {
            matches!(**b, InvokeError::MemoryAccessError)
        } else {
            false
        }
    });
}

#[test]
fn overflow_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package = test_runner.extract_and_publish_package("package");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package, "MaxReturnSize", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        if let RuntimeError::KernelError(KernelError::InvokeError(b)) = e {
            matches!(**b, InvokeError::MemoryAccessError)
        } else {
            false
        }
    });
}

#[test]
fn zero_return_len_should_cause_data_validation_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package = test_runner.extract_and_publish_package("package");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package, "ZeroReturnSize", "f", args!())
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| matches!(e, RuntimeError::KernelError(KernelError::InvokeError(_))));
}

#[test]
fn test_basic_package() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let package = Package {
        code,
        blueprints: HashMap::new(),
    };
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .publish_package(package)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_basic_package_missing_export() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .publish_package(package)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::MissingExport { .. })
            ))
        )
    });
}
