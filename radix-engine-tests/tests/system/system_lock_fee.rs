use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{AttachedModuleId, LockFlags, SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::resource::{NativeFungibleVault, ResourceManager};
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

#[test]
fn cannot_lock_fee_on_new_global_vault() {
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
                "lock_fee" => {
                    let handle =
                        api.actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only())?;
                    let own: Own = api.field_read_typed(handle)?;
                    Vault(own).lock_fee(api, Decimal::one())?;
                }
                "new" => {
                    let metadata = Metadata::create(api)?;
                    let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                    let resman: ResourceManager = ResourceManager(XRD);
                    let vault = resman.new_empty_vault(api)?;
                    let node_id = api.new_simple_object(
                        BLUEPRINT_NAME,
                        indexmap![0u8 => FieldValue::new(vault)],
                    )?;
                    let address = api.globalize(
                        node_id,
                        indexmap!(
                            AttachedModuleId::Metadata => metadata.0,
                            AttachedModuleId::RoleAssignment => access_rules.0.0,
                        ),
                        None,
                    )?;

                    api.call_method(
                        address.as_node_id(),
                        "lock_fee",
                        scrypto_encode(&()).unwrap(),
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
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![("lock_fee", "lock_fee", true), ("new", "new", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::OpenSubstateError(
                    OpenSubstateError::LockUnmodifiedBaseOnNewSubstate(..)
                )
            ))
        )
    })
}
