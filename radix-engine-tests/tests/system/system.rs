use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use radix_substate_store_queries::typed_substate_layout::{RoleAssignmentError, PACKAGE_BLUEPRINT};
use scrypto_test::prelude::*;

#[test]
fn test_handle_mismatch() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "HandleMismatchTest",
            "new",
            manifest_args!(),
        )
        .build();
    let component_address = ledger
        .execute_manifest(manifest, vec![])
        .expect_commit_success()
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "treat_field_handle_as_kv_store_handle",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::NotAKeyValueEntryWriteHandle)
        )
    });
}

#[test]
fn test_put_address_reservation_into_component_state() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .allocate_global_address(PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, "reservation", "address")
        .call_function_with_name_lookup(
            package_address,
            "AddressReservationTest",
            "put_address_reservation_into_component_state",
            |lookup| manifest_args!(lookup.address_reservation("reservation")),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn test_put_address_reservation_into_kv_store() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .allocate_global_address(PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, "reservation", "address")
        .call_function_with_name_lookup(
            package_address,
            "AddressReservationTest",
            "put_address_reservation_into_kv_store",
            |lookup| manifest_args!(lookup.address_reservation("reservation")),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn test_globalize_address_reservation() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .allocate_global_address(PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, "reservation", "address")
        .call_function_with_name_lookup(
            package_address,
            "AddressReservationTest",
            "globalize_address_reservation",
            |lookup| manifest_args!(lookup.address_reservation("reservation")),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn test_write_after_locking_field_substate() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "WriteAfterLockingTest",
            "write_after_locking_field_substate",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(_))
        )
    })
}

#[test]
fn test_write_after_locking_key_value_store_entry() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "WriteAfterLockingTest",
            "write_after_locking_key_value_store_entry",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
        )
    })
}

#[test]
fn test_write_after_locking_key_value_collection_entry() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "WriteAfterLockingTest",
            "write_after_locking_key_value_collection_entry",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
        )
    })
}

#[test]
fn test_set_role_of_role_assignment() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoleAndRole",
            "set_role_of_role_assignment",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                RoleAssignmentError::UsedReservedSpace
            ))
        )
    })
}

#[test]
fn test_set_role_of_role_assignment_v2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoleAndRole",
            "set_role_of_role_assignment_v2",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                RoleAssignmentError::UsedReservedSpace { .. }
            ))
        )
    });
}

#[test]
fn test_call_role_assignment_method_of_role_assignment() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("system"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoleAndRole",
            "call_role_assignment_method_of_role_assignment",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::ObjectModuleDoesNotExist(_))
        )
    });
}
