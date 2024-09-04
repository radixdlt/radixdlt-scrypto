use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError, SystemModuleError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::key_value_store_api::KeyValueStoreDataSchema;
use radix_engine_interface::api::{LockFlags, SystemApi};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

const BLUEPRINT_NAME: &str = "MyBlueprint";
const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
#[derive(Clone)]
struct TestInvoke;
impl VmInvoke for TestInvoke {
    fn invoke<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    >(
        &mut self,
        export_name: &str,
        _input: &IndexedScryptoValue,
        api: &mut Y,
        _vm_api: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            "test" => {
                let kv_store = api.key_value_store_new(
                    KeyValueStoreDataSchema::new_local_with_self_package_replacement::<String, ()>(
                        TEST_UTILS_PACKAGE,
                        false,
                    ),
                )?;
                let long_key = "a".repeat(MAX_SUBSTATE_KEY_SIZE + 1);
                api.key_value_store_open_entry(
                    &kv_store,
                    &scrypto_encode(&long_key).unwrap(),
                    LockFlags::read_only(),
                )?;
            }
            "invalid_schema" => {
                let mut schema = KeyValueStoreDataSchema::new_local_with_self_package_replacement::<
                    String,
                    (),
                >(TEST_UTILS_PACKAGE, false);
                match &mut schema {
                    KeyValueStoreDataSchema::Local {
                        additional_schema, ..
                    } => {
                        additional_schema
                            .v1_mut()
                            .type_metadata
                            .push(TypeMetadata::unnamed());
                    }
                    _ => {}
                }
                api.key_value_store_new(schema)?;
            }
            _ => {}
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

#[test]
fn opening_long_substate_key_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("test", "test", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxSubstateKeySizeExceeded(..)
            ))
        )
    });
}

#[test]
fn kv_store_with_invalid_schema_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("invalid_schema", "invalid_schema", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(
                package_address,
                BLUEPRINT_NAME,
                "invalid_schema",
                manifest_args!(),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidGenericArgs)
        )
    });
}
