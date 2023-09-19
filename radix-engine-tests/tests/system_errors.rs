
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::{
    AttachedModuleId, ClientApi, LockFlags,
};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

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
            "invalid_state_handle" => {
                api.actor_open_field(2u32, 0u8, LockFlags::read_only())?;
            }
            "invalid_ref_handle" => {
                api.actor_get_node_id(9u32)?;
            }
            "invalid_address_reservation" => {
                let self_node_id = api.actor_get_node_id(ACTOR_REF_SELF)?;
                api.get_reservation_address(&self_node_id)?;
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

fn run<F: FnOnce(TransactionReceipt)>(method: &str, on_receipt: F) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![
                ("new", "new", false),
                ("invalid_state_handle", "invalid_state_handle", true),
                ("invalid_ref_handle", "invalid_ref_handle", true),
                ("invalid_address_reservation", "invalid_address_reservation", true),
            ],
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
            .call_method(component_address, method, manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    on_receipt(receipt);
}

#[test]
fn invalid_actor_state_handle_should_error() {
    run("invalid_state_handle", |receipt| {
        receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(SystemError::InvalidActorStateHandle)));
    });
}

#[test]
fn invalid_actor_ref_handle_should_error() {
    run("invalid_ref_handle", |receipt| {
        receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(SystemError::InvalidActorRefHandle)));
    });
}

#[test]
fn invalid_address_reservation_should_error() {
    run("invalid_address_reservation", |receipt| {
        receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(SystemError::NotAnAddressReservation)));
    });
}
