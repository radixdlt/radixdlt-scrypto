use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::{ClientApi, FieldValue, LockFlags, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_store_reference_in_non_transient_blueprint() {
    // Arrange
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            _input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        {
            match export_name {
                "new" => {
                    let auth_zone = api.get_auth_zone()?;
                    let node_id = api.new_simple_object(
                        BLUEPRINT_NAME,
                        vec![FieldValue::new(Reference(auth_zone))],
                    )?;
                    api.drop_object(&node_id)?;
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("new", "new", false)],
        ),
    );

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::InvalidReference))
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
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            _input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        {
            match export_name {
                "new" => {
                    let node_id =
                        api.new_simple_object(BLUEPRINT_NAME, vec![FieldValue::new(())])?;
                    api.call_method(&node_id, "test", scrypto_encode(&()).unwrap())?;
                    api.drop_object(&node_id)?;
                }
                "test" => {
                    let handle =
                        api.actor_open_field(OBJECT_HANDLE_SELF, 0u8, LockFlags::MUTABLE)?;
                    let auth_zone = api.get_auth_zone()?;
                    api.field_write_typed(handle, Reference(auth_zone))?;
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("new", "new", false), ("test", "test", true)],
        ),
    );

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::InvalidReference))
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
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            _input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        {
            match export_name {
                "kv_store" => {
                    let kv_store = api
                        .key_value_store_new(KeyValueStoreGenericArgs::new::<(), Reference>(
                            false,
                        ))?;
                    let handle = api.key_value_store_open_entry(
                        &kv_store,
                        &scrypto_encode(&()).unwrap(),
                        LockFlags::MUTABLE,
                    )?;
                    let auth_zone = api.get_auth_zone()?;
                    api.key_value_entry_set_typed(handle, Reference(auth_zone))?;
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("kv_store", "kv_store", false)],
        ),
    );

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
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
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::InvalidReference))
    });
}
