use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::CreateFrameError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::{ClientApi, ACTOR_REF_AUTH_ZONE};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn should_not_be_able_to_move_auth_zone() {
    // Arrange
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        {
            match export_name {
                "test" => {
                    let auth_zone_id = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE).unwrap();
                    let self_blueprint_id = api.actor_get_blueprint_id()?;
                    api.call_function(
                        self_blueprint_id.package_address,
                        self_blueprint_id.blueprint_name.as_str(),
                        "hi",
                        scrypto_encode(&Own(auth_zone_id)).unwrap(),
                    )?;
                }
                "hi" => {
                    return Ok(input.clone());
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
            vec![("test", "test", false), ("hi", "hi", false)],
        ),
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
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(..))
            ))
        )
    });
}

#[test]
fn test_auth_zone_create_proof_of_all_for_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, 10)
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .drop_proof("proof")
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_auth_zone_create_proof_of_all_for_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            &btreeset!(
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2)
            ),
        )
        .create_proof_from_auth_zone_of_all(resource_address, "proof")
        .drop_proof("proof")
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
