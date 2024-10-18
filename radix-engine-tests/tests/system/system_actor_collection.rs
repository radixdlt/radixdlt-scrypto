use radix_common::prelude::*;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{AttachedModuleId, LockFlags, SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

#[test]
fn opening_read_only_key_value_entry_should_not_create_substates() {
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
                    let _handle = api.actor_open_key_value_entry(
                        ACTOR_STATE_SELF,
                        0u8,
                        &scrypto_encode(&()).unwrap(),
                        LockFlags::read_only(),
                    )?;
                }
                "new" => {
                    let metadata = Metadata::create(api)?;
                    let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, indexmap![])?;
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
        PackageDefinition::new_with_kv_collection_test_definition(
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
    let result = receipt.expect_commit_success();
    let substate_updates = result
        .state_updates
        .clone()
        .into_flattened_substate_updates();
    for ((node_id, _partition_num, _key), update) in substate_updates {
        if matches!(update, DatabaseUpdate::Set(..)) && node_id.eq(component_address.as_node_id()) {
            panic!("No database writes to the component should have occurred");
        }
    }
}
