use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::{
    ClientApi, LockFlags, ObjectModuleId, OBJECT_HANDLE_OUTER_OBJECT,
};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn create_access_rule_of_depth(depth: usize) -> AccessRule {
    let mut rule_node = AccessRuleNode::AnyOf(vec![]);
    for _ in 0..depth {
        rule_node = AccessRuleNode::AnyOf(vec![rule_node]);
    }

    AccessRule::Protected(rule_node)
}

#[test]
fn opening_non_existent_outer_object_fields_should_not_panic() {
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
                "create_access_rule" => {
                    let access_rule = create_access_rule_of_depth(16);
                    RoleAssignment::create(OwnerRole::Fixed(access_rule), btreemap!(), api)?;
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
            vec![("create_access_rule", "create_access_rule", false)],
        ),
    );
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "create_access_rule", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_commit_success();
}
