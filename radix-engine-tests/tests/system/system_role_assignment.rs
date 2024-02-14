use native_sdk::modules::role_assignment::{RoleAssignment, RoleAssignmentObject};
use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::object_modules::role_assignment::RoleAssignmentError;
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_common::constants::AuthAddresses;
use radix_engine_common::prelude::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    PackageDefinition, PackagePublishNativeManifestInput, PACKAGE_BLUEPRINT,
    PACKAGE_PUBLISH_NATIVE_IDENT,
};
use scrypto_test::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::model::{DynamicPackageAddress, InstructionV1};

#[test]
fn cannot_define_more_than_50_roles() {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y, V>(
            &mut self,
            _export_name: &str,
            _input: &IndexedScryptoValue,
            _api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        {
            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let mut roles = index_map_new();
    for i in 0..(MAX_ROLES + 1) {
        roles.insert(RoleKey::new(format!("role{}", i)), RoleList::none());
    }

    // Act
    let receipt = test_runner.execute_system_transaction(
        vec![InstructionV1::CallFunction {
            package_address: DynamicPackageAddress::Static(PACKAGE_PACKAGE),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                definition: PackageDefinition::new_roles_only_test_definition(
                    BLUEPRINT_NAME,
                    roles
                ),
                native_package_code_id: CUSTOM_PACKAGE_CODE_ID,
                metadata: MetadataInit::default(),
                package_address: None,
            }),
        }],
        btreeset!(AuthAddresses::system_role()),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ExceededMaxRoles { .. }
            ))
        )
    });
}

#[test]
fn cannot_define_role_name_larger_than_max() {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y, V>(
            &mut self,
            _export_name: &str,
            _input: &IndexedScryptoValue,
            _api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        {
            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let mut roles = index_map_new();
    roles.insert(
        RoleKey::new("a".repeat(MAX_ROLE_NAME_LEN + 1)),
        RoleList::none(),
    );

    // Act
    let receipt = test_runner.execute_system_transaction(
        vec![InstructionV1::CallFunction {
            package_address: DynamicPackageAddress::Static(PACKAGE_PACKAGE),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                definition: PackageDefinition::new_roles_only_test_definition(
                    BLUEPRINT_NAME,
                    roles
                ),
                native_package_code_id: CUSTOM_PACKAGE_CODE_ID,
                metadata: MetadataInit::default(),
                package_address: None,
            }),
        }],
        btreeset!(AuthAddresses::system_role()),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ExceededMaxRoleNameLen { .. }
            ))
        )
    });
}

#[test]
fn cannot_setup_more_than_50_roles() {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y, V>(
            &mut self,
            export_name: &str,
            _input: &IndexedScryptoValue,
            api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        {
            match export_name {
                "test" => {
                    let mut data = index_map_new();
                    for i in 0..(MAX_ROLES + 1) {
                        data.insert(RoleKey::new(format!("role{}", i)), None);
                    }

                    let _ = RoleAssignment::create(
                        OwnerRole::None,
                        indexmap!(ModuleId::Main => RoleAssignmentInit {
                            data
                        }),
                        api,
                    )?;
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
            vec![("test", "test", false)],
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
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                RoleAssignmentError::ExceededMaxRoles
            ))
        )
    });
}

#[test]
fn cannot_set_role_before_attachment() {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y, V>(
            &mut self,
            export_name: &str,
            _input: &IndexedScryptoValue,
            api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        {
            match export_name {
                "test" => {
                    let role_assignment =
                        RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                    role_assignment.set_role(ModuleId::Main, "test", AccessRule::AllowAll, api)?;
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
            vec![("test", "test", false)],
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
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                RoleAssignmentError::CannotSetRoleIfNotAttached
            ))
        )
    });
}
