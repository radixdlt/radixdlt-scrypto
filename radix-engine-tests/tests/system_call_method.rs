use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::{CreateFrameError, PassMessageError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn call_method_with_owned_actor_should_fail() {
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
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, vec![])?;
                    let args = scrypto_encode(&Own(node_id)).unwrap();

                    let _ = api.call_method(&node_id, "test", args)?;
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
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("test", "test", true), ("new", "new", false)],
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
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(
                    PassMessageError::ActorRefNotFound(..)
                ))
            ))
        )
    });
}
