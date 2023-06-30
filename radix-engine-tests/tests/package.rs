use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{
    ApplicationError, RuntimeError, SystemError, SystemModuleError, VmError,
};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, PackageDefinition,
    PackagePublishNativeManifestInput, PACKAGE_BLUEPRINT,
};
use radix_engine_interface::metadata_init;
use radix_engine_interface::schema::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, BlueprintSchemaInit,
    BlueprintStateSchemaInit, FieldSchema, FunctionSchemaInit, TypeRef,
};
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .publish_package_advanced(
            None,
            code,
            PackageDefinition::default(),
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
                    InvalidMemory::MissingMemorySection
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .publish_package_advanced(
            None,
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
        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            feature_set: btreeset!(),
            dependencies: btreeset!(),

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema: ScryptoSchema {
                    type_kinds: vec![],
                    type_metadata: vec![],
                    type_validations: vec![],
                },
                state: BlueprintStateSchemaInit {
                    fields: vec![FieldSchema::static_field(LocalTypeIndex::WellKnown(
                        UNIT_ID,
                    ))],
                    collections: vec![],
                },
                events: BlueprintEventSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit {
                    functions: btreemap!(
                        "f".to_string() => FunctionSchemaInit {
                            receiver: Option::None,
                            input: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_ID)),
                            output: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_ID)),
                            export: "not_exist".to_string(),
                        }
                    ),
                    virtual_lazy_load_functions: btreemap!(),
                },
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig::default(),
        },
    );
    // Act
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .publish_package_advanced(
            None,
            code,
            PackageDefinition { blueprints },
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

// FIXME: Change test to check that schema type_index is viable
#[test]
fn bad_function_schema_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package, "BadFunctionSchema", "f", manifest_args!())
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(..))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_wasm_package_outside_of_transaction_processor() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/publish_package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package,
            "PublishPackage",
            "publish_package",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_advanced_wasm_package_outside_of_transaction_processor() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/publish_package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package,
            "PublishPackage",
            "publish_package_advanced",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_native_packages() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            "publish_native",
            to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: None,
                native_package_code_id: 0u64,
                setup: PackageDefinition::default(),
                metadata: metadata_init!(),
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_native_packages_in_scrypto() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/publish_package");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package,
            "PublishPackage",
            "publish_native",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}
