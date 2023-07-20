use radix_engine::blueprints::resource::ResourceNativePackage;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{PackageDefinition, RESOURCE_CODE_ID};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn global_address_access_from_frame_owned_object_should_not_succeed() {
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
                    let _ = api.call_method(&node_id, "get_global_address", scrypto_args!())?;
                    let _ = api.drop_object(&node_id)?;
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                "get_global_address" => {
                    let _ = api.actor_get_global_address()?;
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                _ => Ok(IndexedScryptoValue::from_typed(&())),
            }
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![
                ("test", "test", false),
                ("get_global_address", "get_global_address", true),
            ],
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
            RuntimeError::SystemError(SystemError::GlobalAddressDoesNotExist,)
        )
    });
}

#[test]
fn global_address_access_from_direct_access_methods_should_fail_even_with_borrowed_reference() {
    // Arrange
    let resource_direct_access_methods: HashSet<String> = ResourceNativePackage::definition()
        .blueprints
        .into_iter()
        .flat_map(|(_, def)| def.schema.functions.functions.into_iter())
        .filter_map(|(_, def)| {
            def.receiver.and_then(|i| {
                if matches!(i.ref_types, RefTypes::DIRECT_ACCESS) {
                    Some(def.export)
                } else {
                    None
                }
            })
        })
        .collect();
    #[derive(Clone)]
    struct ResourceOverride(HashSet<String>);
    impl VmInvoke for ResourceOverride {
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        {
            if self.0.contains(export_name) {
                api.actor_get_global_address()
                    .expect_err("Direct method calls should never have global address");
            }
            ResourceNativePackage::invoke_export(export_name, input, api)
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(
            RESOURCE_CODE_ID,
            ResourceOverride(resource_direct_access_methods),
        ))
        .build();

    let (public_key, _, account) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish("./tests/blueprints/recall");
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, "RecallTest", "new", manifest_args!())
            .build(),
        vec![],
    );
    let (component_address, recallable): (ComponentAddress, ResourceAddress) =
        receipt.expect_commit_with_success(true).output(1);
    let vault_id = test_runner.get_component_vaults(component_address, recallable)[0];

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_method(
                component_address,
                "recall_on_direct_access_ref_method",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
