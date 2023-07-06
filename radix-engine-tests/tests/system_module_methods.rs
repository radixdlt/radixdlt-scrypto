use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::VmInvoke;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn should_not_be_able_to_call_metadata_methods_on_frame_owned_object() {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
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
                "test" => {
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, vec![])?;
                    let _ = api.call_method_advanced(
                        &node_id,
                        ObjectModuleId::Metadata,
                        false,
                        METADATA_SET_IDENT,
                        scrypto_encode(&MetadataSetInput {
                            key: "key".to_string(),
                            value: MetadataValue::String("value".to_string()),
                        })
                        .unwrap(),
                    )?;
                    api.drop_object(&node_id)?;
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                _ => Ok(IndexedScryptoValue::from_typed(&())),
            }
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .build_with_native_vm(TestNativeVm::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke));
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_test_definition(BLUEPRINT_NAME, vec![("test", "test", false)]),
    );

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::ObjectModuleDoesNotExist(
                ObjectModuleId::Metadata
            ))
        )
    });
}
