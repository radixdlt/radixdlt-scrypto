use radix_blueprint_schema_init::*;
use radix_common::prelude::*;
use radix_engine::blueprints::resource::ResourceNativePackage;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{SystemApi, ACTOR_REF_GLOBAL};
use radix_engine_interface::blueprints::package::{NativeCodeId, PackageDefinition};
use radix_engine_tests::common::*;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

#[test]
fn global_address_access_from_frame_owned_object_should_not_succeed() {
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
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, indexmap!())?;
                    let _ = api.call_method(&node_id, "get_global_address", scrypto_args!())?;
                    let _ = api.drop_object(&node_id)?;
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                "get_global_address" => {
                    let _ = api.actor_get_node_id(ACTOR_REF_GLOBAL)?;
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
            vec![
                ("test", "test", false),
                ("get_global_address", "get_global_address", true),
            ],
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
        fn invoke<
            Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        >(
            &mut self,
            export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError> {
            if self.0.contains(export_name) {
                api.actor_get_node_id(ACTOR_REF_GLOBAL)
                    .expect_err("Direct method calls should never have global address");
            }
            ResourceNativePackage::invoke_export(export_name, input, api)
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(
            NativeCodeId::ResourceCode1 as u64,
            ResourceOverride(resource_direct_access_methods),
        ))
        .build();

    let (public_key, _, account) = ledger.new_allocated_account();

    let package_address = ledger.publish_package_simple(PackageLoader::get("recall"));
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, "RecallTest", "new", manifest_args!())
            .build(),
        vec![],
    );
    let (component_address, recallable): (ComponentAddress, ResourceAddress) =
        receipt.expect_commit(true).output(1);
    let vault_id = ledger.get_component_vaults(component_address, recallable)[0];

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_method(
                component_address,
                "recall_on_direct_access_ref_method",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
