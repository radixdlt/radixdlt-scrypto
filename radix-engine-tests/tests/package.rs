use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{ApplicationError, KernelError, RuntimeError};
use radix_engine::types::*;
use radix_engine::wasm::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema};
use sbor::basic_well_known_types::{ANY_ID, UNIT_ID};
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
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package_advanced(
            code,
            PackageSchema::default(),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRulesConfig::new(),
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
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package, "LargeReturnSize", "f", manifest_args!())
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
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package, "MaxReturnSize", "f", manifest_args!())
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
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package, "ZeroReturnSize", "f", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::InterpreterError(_)));
}

#[test]
fn test_basic_package() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package_advanced(
            code,
            single_function_package_schema("Test", "f"),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRulesConfig::new(),
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
    let mut package_schema = PackageSchema::default();
    package_schema.blueprints.insert(
        "Test".to_string(),
        BlueprintSchema {
            parent: None,
            schema: ScryptoSchema {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            },
            substates: vec![LocalTypeIndex::WellKnown(UNIT_ID)],
            functions: btreemap!(
                "f".to_string() => FunctionSchema {
                    receiver: Option::None,
                    input: LocalTypeIndex::WellKnown(ANY_ID),
                    output: LocalTypeIndex::WellKnown(ANY_ID),
                    export_name: "not_exist".to_string(),
                }
            ),
            virtual_lazy_load_functions: btreemap!(),
            event_schema: [].into(),
        },
    );
    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package_advanced(
            code,
            package_schema,
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRulesConfig::new(),
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
