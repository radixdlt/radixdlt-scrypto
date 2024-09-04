use radix_common::prelude::*;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{
    AttachedModuleId, LockFlags, SystemApi, ACTOR_STATE_OUTER_OBJECT,
};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

#[test]
fn opening_non_existent_outer_object_fields_should_not_panic() {
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
                "test" => {
                    api.actor_open_field(ACTOR_STATE_OUTER_OBJECT, 0u8, LockFlags::read_only())?;
                }
                "new" => {
                    let metadata = Metadata::create(api)?;
                    let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, indexmap!())?;
                    api.globalize(
                        node_id,
                        indexmap!(
                            AttachedModuleId::Metadata => metadata.0,
                            AttachedModuleId::RoleAssignment => access_rules.0.0,
                        ),
                        None,
                    )?;
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
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("test", "test", true), ("new", "new", false)],
        ),
    );
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );
    let component_address = receipt.expect_commit_success().new_component_addresses()[0];

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_method(component_address, "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_failure();
}
