use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::key_value_store_api::KeyValueStoreDataSchema;
use radix_engine_interface::api::{
    FieldValue, LockFlags, SystemApi, ACTOR_REF_AUTH_ZONE, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::package::{KeyOrValue, PackageDefinition};
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

#[test]
fn cannot_store_reference_in_non_transient_blueprint() {
    // Arrange
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
                "new" => {
                    let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
                    let node_id = api.new_simple_object(
                        BLUEPRINT_NAME,
                        indexmap![0u8 => FieldValue::new(Reference(auth_zone))],
                    )?;
                    api.drop_object(&node_id)?;
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("new", "new", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::BlueprintPayloadValidationError(.., error),
        )) => error.contains("Non Global Reference"),
        _ => false,
    });
}

#[test]
fn cannot_write_reference_in_non_transient_blueprint() {
    // Arrange
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
                "new" => {
                    let node_id = api
                        .new_simple_object(BLUEPRINT_NAME, indexmap!(0u8 =>FieldValue::new(())))?;
                    api.call_method(&node_id, "test", scrypto_encode(&()).unwrap())?;
                    api.drop_object(&node_id)?;
                }
                "test" => {
                    let handle = api.actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE)?;
                    let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
                    api.field_write_typed(handle, &Reference(auth_zone))?;
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("new", "new", false), ("test", "test", true)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::BlueprintPayloadValidationError(.., error),
        )) => error.contains("Non Global Reference"),
        _ => false,
    });
}

#[test]
fn cannot_write_reference_in_kv_store() {
    // Arrange
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
                "kv_store" => {
                    let kv_store = api.key_value_store_new(
                        KeyValueStoreDataSchema::new_local_without_self_package_replacement::<
                            (),
                            Reference,
                        >(false),
                    )?;
                    let handle = api.key_value_store_open_entry(
                        &kv_store,
                        &scrypto_encode(&()).unwrap(),
                        LockFlags::MUTABLE,
                    )?;
                    let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
                    api.key_value_entry_set_typed(handle, Reference(auth_zone))?;
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("kv_store", "kv_store", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(
                package_address,
                BLUEPRINT_NAME,
                "kv_store",
                manifest_args!(),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::KeyValueStorePayloadValidationError(KeyOrValue::Value, error),
        )) => error.contains("Non Global Reference"),
        _ => false,
    });
}
