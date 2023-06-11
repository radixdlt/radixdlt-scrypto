use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{ApplicationError, RuntimeError, VmError};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine_interface::blueprints::package::{
    BlueprintSetup, FunctionSetup, MethodAuthTemplate, PackageSetup,
};
use radix_engine_interface::schema::{BlueprintSchema, FieldSchema};
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
            PackageSetup::default(),
            BTreeMap::new(),
            OwnerRole::None,
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
        if let RuntimeError::VmError(VmError::Wasm(b)) = e {
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
        if let RuntimeError::VmError(VmError::Wasm(b)) = e {
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
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemUpstreamError(_)));
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
            single_function_package_definition("Test", "f"),
            BTreeMap::new(),
            OwnerRole::None,
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
        "Test".to_string(),
        BlueprintSetup {
            outer_blueprint: None,
            dependencies: btreeset!(),
            features: btreeset!(),
            blueprint: BlueprintSchema {
                fields: vec![FieldSchema::normal(LocalTypeIndex::WellKnown(UNIT_ID))],
                collections: vec![],
            },
            event_schema: [].into(),
            schema: ScryptoSchema {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            },
            function_auth: btreemap!(),
            royalty_config: RoyaltyConfig::default(),
            template: MethodAuthTemplate {
                method_auth_template: btreemap!(),
                outer_method_auth_template: btreemap!(),
            },
            virtual_lazy_load_functions: btreemap!(),
            functions: btreemap!(
                "f".to_string() => FunctionSetup {
                    receiver: Option::None,
                    input: LocalTypeIndex::WellKnown(ANY_ID),
                    output: LocalTypeIndex::WellKnown(ANY_ID),
                    export: "not_exist".to_string(),
                }
            ),
        },
    );
    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package_advanced(
            code,
            PackageSetup { blueprints },
            BTreeMap::new(),
            OwnerRole::None,
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
