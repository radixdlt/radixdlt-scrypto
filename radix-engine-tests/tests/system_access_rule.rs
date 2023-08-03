use native_sdk::modules::role_assignment::{RoleAssignment, RoleAssignmentObject};
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::node_modules::role_assignment::RoleAssignmentError;
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn creating_an_owner_access_rule_which_is_beyond_the_depth_limit_should_error() {
    let access_rule = create_access_rule_of_depth(MAX_ACCESS_RULE_DEPTH + 1);
    creating_an_access_rule_which_is_beyond_the_depth_limit_should_error(
        AccessRuleCreation::OwnerCreation,
        access_rule,
        |e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                    RoleAssignmentError::ExceededMaxAccessRuleDepth
                ))
            )
        },
    );
}

#[test]
fn creating_a_regular_access_rule_which_is_beyond_the_depth_limit_should_error() {
    let access_rule = create_access_rule_of_depth(MAX_ACCESS_RULE_DEPTH + 1);
    creating_an_access_rule_which_is_beyond_the_depth_limit_should_error(
        AccessRuleCreation::RoleCreation,
        access_rule,
        |e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                    RoleAssignmentError::ExceededMaxAccessRuleDepth
                ))
            )
        },
    );
}

#[test]
fn setting_an_owner_access_rule_which_is_beyond_the_depth_limit_should_error() {
    let access_rule = create_access_rule_of_depth(MAX_ACCESS_RULE_DEPTH + 1);
    creating_an_access_rule_which_is_beyond_the_depth_limit_should_error(
        AccessRuleCreation::OwnerSet,
        access_rule,
        |e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                    RoleAssignmentError::ExceededMaxAccessRuleDepth
                ))
            )
        },
    );
}

#[test]
fn setting_a_role_access_rule_which_is_beyond_the_depth_limit_should_error() {
    let access_rule = create_access_rule_of_depth(MAX_ACCESS_RULE_DEPTH + 1);
    creating_an_access_rule_which_is_beyond_the_depth_limit_should_error(
        AccessRuleCreation::RoleSet,
        access_rule,
        |e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                    RoleAssignmentError::ExceededMaxAccessRuleDepth
                ))
            )
        },
    );
}

fn create_access_rule_of_depth(depth: usize) -> AccessRule {
    let mut rule_node = AccessRuleNode::AnyOf(vec![]);
    for _ in 0..depth {
        rule_node = AccessRuleNode::AnyOf(vec![rule_node]);
    }

    AccessRule::Protected(rule_node)
}

fn create_access_rule_of_length(size: usize) -> AccessRule {
    let mut nodes = vec![];
    for _ in 0..size {
        nodes.push(AccessRuleNode::AnyOf(vec![]));
    }
    AccessRule::Protected(AccessRuleNode::AllOf(nodes))
}

#[derive(Copy, Clone)]
enum AccessRuleCreation {
    OwnerCreation,
    RoleCreation,
    OwnerSet,
    RoleSet,
}

fn creating_an_access_rule_which_is_beyond_the_depth_limit_should_error<F>(
    access_rule_creation: AccessRuleCreation,
    access_rule: AccessRule,
    check_result: F,
) where
    F: Fn(&RuntimeError) -> bool,
{
    // Arrange
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
    #[derive(Clone)]
    struct TestInvoke(AccessRuleCreation, AccessRule);
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
                "create_access_rule" => match self.0 {
                    AccessRuleCreation::OwnerCreation => {
                        RoleAssignment::create(OwnerRole::Fixed(self.1.clone()), btreemap!(), api)?;
                    }
                    AccessRuleCreation::RoleCreation => {
                        RoleAssignment::create(
                            OwnerRole::None,
                            btreemap!(ObjectModuleId::Main => roles2!("test" => self.1.clone();)),
                            api,
                        )?;
                    }
                    AccessRuleCreation::OwnerSet => {
                        let role_assignment = RoleAssignment::create(
                            OwnerRole::Updatable(AccessRule::AllowAll),
                            btreemap!(),
                            api,
                        )?;
                        role_assignment.set_owner_role(self.1.clone(), api)?;
                    }
                    AccessRuleCreation::RoleSet => {
                        let role_assignment = RoleAssignment::create(
                            OwnerRole::Updatable(AccessRule::AllowAll),
                            btreemap!(),
                            api,
                        )?;
                        role_assignment.set_role(
                            ObjectModuleId::Main,
                            RoleKey::new("test"),
                            self.1.clone(),
                            api,
                        )?;
                    }
                },
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(
            CUSTOM_PACKAGE_CODE_ID,
            TestInvoke(access_rule_creation, access_rule),
        ))
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
            .call_function(
                package_address,
                BLUEPRINT_NAME,
                "create_access_rule",
                manifest_args!(),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(check_result);
}
