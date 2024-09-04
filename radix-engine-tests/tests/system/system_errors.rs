use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{AttachedModuleId, LockFlags, SystemApi};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

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
            "invalid_kv_store" => {
                let self_node_id = api.actor_get_node_id(ACTOR_REF_SELF)?;
                api.key_value_store_open_entry(
                    &self_node_id,
                    &scrypto_encode(&()).unwrap(),
                    LockFlags::read_only(),
                )?;
            }
            "invalid_actor_node_id" => {
                api.actor_get_node_id(ACTOR_REF_SELF)?;
            }
            "invalid_outer_object" => {
                let self_node_id = api.actor_get_node_id(ACTOR_REF_SELF)?;
                api.get_outer_object(&self_node_id)?;
            }
            "invalid_field" => {
                api.actor_open_field(ACTOR_STATE_SELF, 4, LockFlags::read_only())?;
            }
            "invalid_collection" => {
                api.actor_open_key_value_entry(
                    ACTOR_STATE_SELF,
                    4,
                    &scrypto_encode(&()).unwrap(),
                    LockFlags::read_only(),
                )?;
            }
            "mutate_immutable_field" => {
                let handle = api.actor_open_field(ACTOR_STATE_SELF, 0, LockFlags::MUTABLE)?;
                api.field_lock(handle)?;
                api.field_close(handle)?;
                api.actor_open_field(ACTOR_STATE_SELF, 0, LockFlags::MUTABLE)?;
            }
            "invalid_kv_entry_handle" => {
                let handle = api.actor_open_field(ACTOR_STATE_SELF, 0, LockFlags::MUTABLE)?;
                api.key_value_entry_get(handle)?;
            }
            "invalid_event_flags" => {
                api.actor_emit_event(
                    "event".to_string(),
                    scrypto_encode(&()).unwrap(),
                    EventFlags::FORCE_WRITE,
                )?;
            }
            "new" => {
                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                let node_id =
                    api.new_simple_object(BLUEPRINT_NAME, indexmap!(0u8 => FieldValue::new(())))?;
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

fn run<F: FnOnce(TransactionReceipt)>(method: &str, is_method: bool, on_receipt: F) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_with_field_test_definition(
            BLUEPRINT_NAME,
            vec![
                ("new", "new", false),
                ("invalid_state_handle", "invalid_state_handle", true),
                ("invalid_ref_handle", "invalid_ref_handle", true),
                (
                    "invalid_address_reservation",
                    "invalid_address_reservation",
                    true,
                ),
                ("invalid_kv_store", "invalid_kv_store", true),
                ("invalid_actor_node_id", "invalid_actor_node_id", false),
                ("invalid_outer_object", "invalid_outer_object", true),
                ("invalid_field", "invalid_field", true),
                ("invalid_collection", "invalid_collection", true),
                ("mutate_immutable_field", "mutate_immutable_field", true),
                ("invalid_kv_entry_handle", "invalid_kv_entry_handle", true),
                ("invalid_event_flags", "invalid_event_flags", true),
            ],
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
    let receipt = if is_method {
        ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(ledger.faucet_component(), 500u32)
                .call_method(component_address, method, manifest_args!())
                .build(),
            vec![],
        )
    } else {
        ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(ledger.faucet_component(), 500u32)
                .call_function(package_address, BLUEPRINT_NAME, method, manifest_args!())
                .build(),
            vec![],
        )
    };

    // Assert
    on_receipt(receipt);
}

#[test]
fn invalid_actor_state_handle_should_error() {
    run("invalid_state_handle", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::InvalidActorStateHandle)
            )
        });
    });
}

#[test]
fn invalid_actor_ref_handle_should_error() {
    run("invalid_ref_handle", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::InvalidActorRefHandle)
            )
        });
    });
}

#[test]
fn invalid_address_reservation_should_error() {
    run("invalid_address_reservation", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::NotAnAddressReservation)
            )
        });
    });
}

#[test]
fn invalid_key_value_store_should_error() {
    run("invalid_kv_store", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(e, RuntimeError::SystemError(SystemError::NotAKeyValueStore))
        });
    });
}

#[test]
fn invalid_actor_node_id_should_error() {
    run("invalid_actor_node_id", false, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::ActorNodeIdDoesNotExist)
            )
        });
    });
}

#[test]
fn invalid_outer_object_should_error() {
    run("invalid_outer_object", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::OuterObjectDoesNotExist)
            )
        });
    });
}

#[test]
fn invalid_field_should_error() {
    run("invalid_field", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::FieldDoesNotExist(..))
            )
        });
    });
}

#[test]
fn invalid_collection_should_error() {
    run("invalid_collection", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::CollectionIndexDoesNotExist(..))
            )
        });
    });
}

#[test]
fn mutating_immutable_field_should_error() {
    run("mutate_immutable_field", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(e, RuntimeError::SystemError(SystemError::FieldLocked(..)))
        });
    });
}

#[test]
fn invalid_key_value_entry_handle_should_error() {
    run("invalid_kv_entry_handle", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::NotAKeyValueEntryHandle)
            )
        });
    });
}

#[test]
fn invalid_event_flags() {
    run("invalid_event_flags", true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::ForceWriteEventFlagsNotAllowed)
            )
        });
    });
}
