use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{
    AttachedModuleId, FieldValue, LockFlags, SystemApi, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::object_modules::royalty::{
    ComponentRoyaltySetInput, COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

fn should_not_be_able_to_call_royalty_methods(resource: bool) {
    // Arrange
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
            _export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError> {
            let node_id = input.references()[0];
            let _ = api.call_module_method(
                &node_id,
                AttachedModuleId::Royalty,
                COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
                scrypto_encode(&ComponentRoyaltySetInput {
                    method: "some_method".to_string(),
                    amount: RoyaltyAmount::Free,
                })
                .unwrap(),
            )?;

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
            vec![("test", "test", false)],
        ),
    );

    let args = if resource {
        let resource_address = ledger
            .create_everything_allowed_non_fungible_resource(OwnerRole::Fixed(rule!(allow_all)));
        manifest_args!(resource_address)
    } else {
        manifest_args!(package_address)
    };

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "test", args)
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::ObjectModuleDoesNotExist(
                AttachedModuleId::Royalty
            ))
        )
    });
}

#[test]
fn should_not_be_able_to_call_royalty_methods_on_resource_manager() {
    should_not_be_able_to_call_royalty_methods(true);
}

#[test]
fn should_not_be_able_to_call_royalty_methods_on_package() {
    should_not_be_able_to_call_royalty_methods(false);
}

#[test]
fn should_not_be_able_to_call_metadata_methods_on_frame_owned_object() {
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
                    let node_id = api.new_simple_object(BLUEPRINT_NAME, indexmap![])?;
                    let _ = api.call_module_method(
                        &node_id,
                        AttachedModuleId::Metadata,
                        METADATA_SET_IDENT,
                        scrypto_encode(&MetadataSetInput {
                            key: "key".to_string(),
                            value: MetadataValue::String("value".to_string()),
                        })
                        .unwrap(),
                    )?;
                    api.drop_object(&node_id)?;
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
            RuntimeError::SystemError(SystemError::ObjectModuleDoesNotExist(
                AttachedModuleId::Metadata
            ))
        )
    });
}

fn should_not_be_able_to_call_metadata_methods_on_child_object(globalized_parent: bool) {
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

    // Arrange
    #[derive(Clone)]
    struct TestInvoke {
        globalized_parent: bool,
    }
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
                    let child = api.new_simple_object(
                        BLUEPRINT_NAME,
                        indexmap! {
                            0u8 => FieldValue::new(&Option::<Own>::None),
                        },
                    )?;
                    let parent = api.new_simple_object(
                        BLUEPRINT_NAME,
                        indexmap! {
                            0u8 => FieldValue::new(&Option::<Own>::Some(Own(child))),
                        },
                    )?;

                    let parent_node_id = if self.globalized_parent {
                        let metadata = Metadata::create(api)?;
                        let role_assignment =
                            RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;

                        let address = api.globalize(
                            parent,
                            indexmap!(
                                AttachedModuleId::Metadata => metadata.0,
                                AttachedModuleId::RoleAssignment => role_assignment.0.0,
                            ),
                            None,
                        )?;
                        address.into_node_id()
                    } else {
                        parent
                    };

                    api.call_method(&parent_node_id, "call_metadata_on_child", scrypto_args!())?;

                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                "call_metadata_on_child" => {
                    let handle =
                        api.actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only())?;
                    let child: Option<Own> = api.field_read_typed(handle)?;

                    let _ = api.call_module_method(
                        &child.unwrap().0,
                        AttachedModuleId::Metadata,
                        METADATA_SET_IDENT,
                        scrypto_encode(&MetadataSetInput {
                            key: "key".to_string(),
                            value: MetadataValue::String("value".to_string()),
                        })
                        .unwrap(),
                    )?;

                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                _ => Ok(IndexedScryptoValue::from_typed(&())),
            }
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(
            CUSTOM_PACKAGE_CODE_ID,
            TestInvoke { globalized_parent },
        ))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![
                ("test", "test", false),
                ("call_metadata_on_child", "call_metadata_on_child", true),
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
            RuntimeError::SystemError(SystemError::ObjectModuleDoesNotExist(
                AttachedModuleId::Metadata
            ))
        )
    });
}

#[test]
fn should_not_be_able_to_call_metadata_methods_on_frame_owned_child_object() {
    should_not_be_able_to_call_metadata_methods_on_child_object(false);
}

#[test]
fn should_not_be_able_to_call_metadata_methods_on_globalized_child_object() {
    should_not_be_able_to_call_metadata_methods_on_child_object(true);
}
