use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::{ClientApi, LockFlags, ACTOR_STATE_SELF, ModuleId};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_store_interface::interface::DatabaseUpdate;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn opening_read_only_key_value_entry_should_not_create_substates() {
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
                    let access_rules = RoleAssignment::create(OwnerRole::None, btreemap!(), api)?;
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, vec![])?;
                    api.globalize(
                        node_id,
                        btreemap!(
                            ModuleId::Metadata => metadata.0,
                            ModuleId::RoleAssignment => access_rules.0.0,
                        ),
                        None,
                    )?;
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
        PackageDefinition::new_with_kv_collection_test_definition(
            BLUEPRINT_NAME,
            vec![("test", "test", true), ("new", "new", false)],
        ),
    );
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );
    let component_address = receipt.expect_commit_success().new_component_addresses()[0];

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_method(component_address, "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    let result = receipt.expect_commit_success();
    for ((node_id, _partition_num), updates) in &result.state_updates.system_updates {
        for (_key, update) in updates {
            if matches!(update, DatabaseUpdate::Set(..))
                && node_id.eq(component_address.as_node_id())
            {
                panic!("No database writes to the component should have occurred");
            }
        }
    }
}
