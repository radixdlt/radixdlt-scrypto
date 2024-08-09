use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltiesInput, PackageDefinition, PACKAGE_CLAIM_ROYALTIES_IDENT,
};
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

#[test]
fn claiming_royalties_on_native_packages_should_be_unauthorized() {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
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
                    api.call_method(
                        PACKAGE_PACKAGE.as_node_id(),
                        PACKAGE_CLAIM_ROYALTIES_IDENT,
                        scrypto_encode(&PackageClaimRoyaltiesInput {}).unwrap(),
                    )?;
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                _ => Ok(IndexedScryptoValue::from_typed(&())),
            }
        }
    }
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
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}
